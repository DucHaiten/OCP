use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocp_sdk::{
    admit_bridge_emit_v1, init_project, poll_bridge_event_v1, resolve_bridge_runtime_plan_v1,
    resolve_domain_selection_v1, run_reactor_service_with_lock, sync_cosmos_lock_v1,
    sync_deps_lock_v1, sync_policy_lock_v1, BridgeRuntimeStateV1, ReactorRuntimeMode,
    ReactorServiceOptions,
};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocp_v5_w2_{tag}_{stamp}"))
}

fn repo_root_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
        .join("..")
}

fn prepare_project(root: &Path) {
    init_project(root).expect("init project");
    let manifest = r#"[package]
name = "v5_w2_domain"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["*"]
deny = []

[policy]
budget_profile = "ci_default"

[runtime]
io_max_events_per_tick = 4
actor_max = 1
"#;
    fs::write(root.join("Ocp.toml"), manifest).expect("write manifest");
    fs::write(
        root.join("src").join("main.ocp"),
        "let ok = true;\ncondition(ok);\n",
    )
    .expect("write source");
    sync_deps_lock_v1(root).expect("sync deps lock");
}

#[test]
fn v5_w2_domain_default_without_first_fallback_pass() {
    let root = temp_project_dir("domain_default");
    prepare_project(&root);
    sync_policy_lock_v1(&root).expect("policy lock");

    let cosmos = r#"version = 1

[[universe]]
id = "ci_locked"
runtime_mode = "deterministic"
engine = "dual"
policy_profile_id = "ci_default"
audit = "hash_only"
trace = "hash_only"

[[domain]]
id = "default"
universe_id = "ci_locked"

[[domain]]
id = "side"
universe_id = "ci_locked"
"#;
    fs::write(root.join("cosmos.toml"), cosmos).expect("write cosmos");

    let selected = resolve_domain_selection_v1(&root, false, Some("ci_locked"), None)
        .expect("resolve default domain");
    assert_eq!(selected.universe_id, "ci_locked");
    assert_eq!(selected.domain_id, "default");
}

#[test]
fn v5_w2_domain_required_without_default_fail_hard() {
    let root = temp_project_dir("domain_required");
    prepare_project(&root);
    sync_policy_lock_v1(&root).expect("policy lock");

    let cosmos = r#"version = 1

[[universe]]
id = "ci_locked"
runtime_mode = "deterministic"
engine = "dual"
policy_profile_id = "ci_default"
audit = "hash_only"
trace = "hash_only"

[[domain]]
id = "alpha"
universe_id = "ci_locked"

[[domain]]
id = "beta"
universe_id = "ci_locked"
"#;
    fs::write(root.join("cosmos.toml"), cosmos).expect("write cosmos");

    let err = resolve_domain_selection_v1(&root, false, Some("ci_locked"), None)
        .expect_err("must require explicit domain");
    assert!(
        err.to_string().contains("V-DOMAIN-REQUIRED"),
        "unexpected error: {err}"
    );
}

#[test]
fn v5_w2_bridge_plan_quota_capacity_and_round_robin_pass() {
    let root = temp_project_dir("bridge_plan");
    prepare_project(&root);
    sync_policy_lock_v1(&root).expect("policy lock");

    let cosmos = r#"version = 1

[[universe]]
id = "ci_locked"
runtime_mode = "deterministic"
engine = "dual"
policy_profile_id = "ci_default"
audit = "hash_only"
trace = "hash_only"

[[domain]]
id = "default"
universe_id = "ci_locked"

[[domain]]
id = "side"
universe_id = "ci_locked"

[[bridge]]
id = "a_default_to_side"
universe_id = "ci_locked"
from_domain = "default"
to_domain = "side"
emit_quota_per_tick = 2
dispatch_max_per_tick = 1
bridge_queue_capacity = 2

[[bridge]]
id = "b_side_to_default"
universe_id = "ci_locked"
from_domain = "side"
to_domain = "default"
emit_quota_per_tick = 1
dispatch_max_per_tick = 1
bridge_queue_capacity = 1
"#;
    fs::write(root.join("cosmos.toml"), cosmos).expect("write cosmos");

    let plan = resolve_bridge_runtime_plan_v1(&root, false, Some("ci_locked"), Some("default"), 3)
        .expect("resolve bridge runtime plan");
    assert_eq!(plan.bridges.len(), 2);
    assert_eq!(
        plan.dispatch_start_index, 1,
        "tick=3 with 2 bridges -> start=1"
    );

    let rule = plan
        .bridges
        .iter()
        .find(|b| b.bridge_id == "a_default_to_side")
        .expect("bridge rule");
    let mut state = BridgeRuntimeStateV1::default();

    admit_bridge_emit_v1(&mut state, rule, 10, "h1").expect("emit #1");
    admit_bridge_emit_v1(&mut state, rule, 10, "h2").expect("emit #2");
    let quota_err = admit_bridge_emit_v1(&mut state, rule, 10, "h3").expect_err("quota must fail");
    assert!(
        quota_err.to_string().contains("V-DOMAIN-BRIDGE-CAP"),
        "unexpected quota err: {quota_err}"
    );

    let first = poll_bridge_event_v1(&mut state, rule, 10).expect("first poll");
    assert_eq!(first.seq, 1);
    let second = poll_bridge_event_v1(&mut state, rule, 10);
    assert!(
        second.is_none(),
        "dispatch_max_per_tick=1 should cap second poll in same tick"
    );

    let third = poll_bridge_event_v1(&mut state, rule, 11).expect("next tick poll");
    assert_eq!(third.seq, 2);
}

#[test]
fn v5_w2_locked_cosmos_requires_bridge_queue_capacity() {
    let root = temp_project_dir("locked_bridge_capacity");
    prepare_project(&root);
    sync_policy_lock_v1(&root).expect("policy lock");
    let repo_root = repo_root_dir();
    let sign_key = repo_root.join("projects/ocp/security/dev-root-1.signing.key.toml");
    let trust_store = repo_root.join("projects/ocp/security/trust.store.toml");

    let cosmos = r#"version = 1

[[universe]]
id = "ci_locked"
runtime_mode = "deterministic"
engine = "dual"
policy_profile_id = "ci_default"
audit = "hash_only"
trace = "hash_only"

[[domain]]
id = "default"
universe_id = "ci_locked"

[[bridge]]
id = "missing_capacity"
universe_id = "ci_locked"
from_domain = "default"
to_domain = "default"
emit_quota_per_tick = 2
dispatch_max_per_tick = 1
"#;
    fs::write(root.join("cosmos.toml"), cosmos).expect("write cosmos");

    let err = sync_cosmos_lock_v1(
        &root,
        true,
        Some("dev-root-1"),
        Some(&sign_key),
        Some(&trust_store),
    )
    .expect_err("locked sync must require bridge_queue_capacity");
    assert!(
        err.to_string().contains("V-BRIDGE-QUEUE-CAPACITY-REQUIRED"),
        "unexpected err: {err}"
    );
}

#[test]
fn v5_w2_reactor_domain_scheduler_partition_pass() {
    let root = temp_project_dir("reactor_domain_partition");
    prepare_project(&root);
    sync_policy_lock_v1(&root).expect("policy lock");
    let repo_root = repo_root_dir();
    let sign_key = repo_root.join("projects/ocp/security/dev-root-1.signing.key.toml");
    let trust_store = repo_root.join("projects/ocp/security/trust.store.toml");

    let source = r#"module tests.v5.w2.reactor;

fn on_event(event) {
  let stable = true;
  condition(stable);
  return event;
}

let ready = true;
condition(ready);
"#;
    fs::write(root.join("src").join("main.ocp"), source).expect("write source");

    let cosmos = r#"version = 1

[[universe]]
id = "ci_locked"
runtime_mode = "deterministic"
engine = "dual"
policy_profile_id = "ci_default"
audit = "hash_only"
trace = "hash_only"

[[domain]]
id = "default"
universe_id = "ci_locked"

[[domain]]
id = "side"
universe_id = "ci_locked"
"#;
    fs::write(root.join("cosmos.toml"), cosmos).expect("write cosmos");
    sync_cosmos_lock_v1(
        &root,
        true,
        Some("dev-root-1"),
        Some(&sign_key),
        Some(&trust_store),
    )
    .expect("cosmos lock sync");

    let report_all = run_reactor_service_with_lock(
        &root,
        &ReactorServiceOptions {
            ticks: 3,
            runtime_mode: ReactorRuntimeMode::Deterministic,
            socket_listen: None,
            runtime_report: None,
            replay_audit: None,
            io_tape_record_path: None,
            io_tape_replay_path: None,
            hive_caps: None,
            universe_id: Some("ci_locked".to_string()),
            domain_id: None,
        },
        true,
    )
    .expect("reactor run all domains");
    assert_eq!(report_all.domain_count, 2);
    assert_eq!(report_all.event_count, 6);
    assert_eq!(report_all.universe_id, "ci_locked");
    assert_eq!(report_all.domain_id, "default");

    let report_single = run_reactor_service_with_lock(
        &root,
        &ReactorServiceOptions {
            ticks: 3,
            runtime_mode: ReactorRuntimeMode::Deterministic,
            socket_listen: None,
            runtime_report: None,
            replay_audit: None,
            io_tape_record_path: None,
            io_tape_replay_path: None,
            hive_caps: None,
            universe_id: Some("ci_locked".to_string()),
            domain_id: Some("side".to_string()),
        },
        true,
    )
    .expect("reactor run single domain");
    assert_eq!(report_single.domain_count, 1);
    assert_eq!(report_single.event_count, 3);
    assert_eq!(report_single.domain_id, "side");
}

#[test]
fn v5_w2_reactor_bridge_backpressure_counter_pass() {
    let root = temp_project_dir("reactor_bridge_backpressure");
    prepare_project(&root);
    sync_policy_lock_v1(&root).expect("policy lock");
    let repo_root = repo_root_dir();
    let sign_key = repo_root.join("projects/ocp/security/dev-root-1.signing.key.toml");
    let trust_store = repo_root.join("projects/ocp/security/trust.store.toml");

    let source = r#"module tests.v5.w2.bridge;

fn on_event(event) {
  let stable = true;
  condition(stable);
  return event;
}

let ready = true;
condition(ready);
"#;
    fs::write(root.join("src").join("main.ocp"), source).expect("write source");

    let cosmos = r#"version = 1

[[universe]]
id = "ci_locked"
runtime_mode = "throughput"
engine = "dual"
policy_profile_id = "ci_default"
audit = "hash_only"
trace = "hash_only"

[[domain]]
id = "default"
universe_id = "ci_locked"

[[bridge]]
id = "default_loop"
universe_id = "ci_locked"
from_domain = "default"
to_domain = "default"
emit_quota_per_tick = 4
dispatch_max_per_tick = 1
bridge_queue_capacity = 1
"#;
    fs::write(root.join("cosmos.toml"), cosmos).expect("write cosmos");
    sync_cosmos_lock_v1(
        &root,
        true,
        Some("dev-root-1"),
        Some(&sign_key),
        Some(&trust_store),
    )
    .expect("cosmos lock sync");

    let report = run_reactor_service_with_lock(
        &root,
        &ReactorServiceOptions {
            ticks: 2,
            runtime_mode: ReactorRuntimeMode::Throughput,
            socket_listen: None,
            runtime_report: None,
            replay_audit: None,
            io_tape_record_path: None,
            io_tape_replay_path: None,
            hive_caps: None,
            universe_id: Some("ci_locked".to_string()),
            domain_id: Some("default".to_string()),
        },
        true,
    )
    .expect("reactor bridge run");
    assert!(
        report.bridge_emit_count > 0,
        "bridge emits should be counted"
    );
    assert!(
        report.bridge_dispatch_count > 0,
        "bridge dispatch should be counted"
    );
    assert!(
        report.bridge_backpressure_count > 0,
        "backpressure should be counted when queue is full"
    );
    assert!(
        report.backpressure_count >= report.bridge_backpressure_count,
        "global backpressure should include bridge backpressure"
    );
}
