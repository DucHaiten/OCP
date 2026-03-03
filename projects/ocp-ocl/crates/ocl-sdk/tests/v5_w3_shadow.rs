use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocl_runtime_core::{CommitPolicyMode, ExecConfig, RunEngine};
use ocl_sdk::{
    compare_shadow_traces_v1, init_project, run_project_with_shadow_compare,
    run_project_with_trace_engine_and_lock, run_project_with_trace_engine_config_and_lock,
    run_reactor_service_with_trace_engine_and_lock, sync_deps_lock_v1, trace_required_digest,
    InputEnvelopeV1, ReactorRuntimeMode, ReactorServiceOptions, SdkError, ShadowOptionsV1,
    ShadowPolicyV1, SHADOW_EMPTY_COMMIT_HASH256,
};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocl_v5_w3_{tag}_{stamp}"))
}

fn prepare_project(root: &Path, source: &str) {
    init_project(root).expect("init project");
    let manifest = r#"[package]
name = "v5_w3_shadow"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["*"]
deny = []
"#;
    fs::write(root.join("Ocl.toml"), manifest).expect("write manifest");
    fs::write(root.join("src").join("main.ocl"), source).expect("write source");
    sync_deps_lock_v1(root).expect("sync deps lock");
}

#[test]
fn v5_shadow_forbid_commit_blocks_effects() {
    let root = temp_project_dir("forbid_commit");
    prepare_project(
        &root,
        r#"observe("world.ok.signal", "tier2", ctx("x=1"), budget(3)) -> seen;
commit(seen);
let ready = true;
condition(ready);
"#,
    );

    let summary = run_project_with_shadow_compare(
        &root,
        RunEngine::Dual,
        true,
        &ShadowOptionsV1 {
            shadow_id: "parity".to_string(),
            policy: ShadowPolicyV1::ForbidCommit,
            report_dir: Some(root.join("target").join("shadow")),
        },
    )
    .expect("shadow compare run");

    assert!(summary.compare.matched, "shadow digest must match");
    assert!(summary.artifacts.report_path.exists());
    assert!(summary.artifacts.main_transcript_path.exists());
    assert!(summary.artifacts.shadow_transcript_path.exists());
    assert!(
        summary
            .compare
            .main_transcript
            .iter()
            .all(|e| e.commit_intents_hash256 == SHADOW_EMPTY_COMMIT_HASH256),
        "forbid_commit must keep commit hash empty"
    );
}

#[test]
fn v5_shadow_digest_matches_when_inputs_equal() {
    let root = temp_project_dir("digest_equal");
    prepare_project(
        &root,
        "let stable = true;\ncondition(stable);\ncondition(stable);\n",
    );

    let summary =
        run_project_with_shadow_compare(&root, RunEngine::Dual, true, &ShadowOptionsV1::default())
            .expect("shadow compare run");

    assert_eq!(
        summary.compare.main_required_digest,
        summary.compare.shadow_required_digest
    );
}

#[test]
fn v5_shadow_mismatch_detected_only_when_contract_valid() {
    let root = temp_project_dir("mismatch_contract");
    prepare_project(&root, "let ok = true;\ncondition(ok);\n");

    let main =
        run_project_with_trace_engine_and_lock(&root, RunEngine::Dual, true).expect("main trace");
    let mut shadow =
        run_project_with_trace_engine_and_lock(&root, RunEngine::Dual, true).expect("shadow trace");
    if let Some(first) = shadow.events.first_mut() {
        first.payload_hash = "aaaaaaaaaaaaaaaa".to_string();
    }

    let envelope = InputEnvelopeV1 {
        universe_id: "__legacy__".to_string(),
        domain_id: "default".to_string(),
        shadow_id: "parity".to_string(),
        runtime_mode: "deterministic".to_string(),
        engine: "dual".to_string(),
        run_kind: "project".to_string(),
    };

    let mismatch = compare_shadow_traces_v1(
        &main.events,
        &shadow.events,
        &envelope,
        ShadowPolicyV1::ForbidCommit,
    )
    .expect("compare report");
    assert!(!mismatch.matched, "mutated transcript must mismatch");

    let unsupported_env = InputEnvelopeV1 {
        runtime_mode: "throughput".to_string(),
        ..envelope
    };
    let unsupported = compare_shadow_traces_v1(
        &main.events,
        &shadow.events,
        &unsupported_env,
        ShadowPolicyV1::ForbidCommit,
    )
    .expect_err("unsupported runtime must fail first");
    match unsupported {
        SdkError::ShadowUnsupported(msg) => assert!(msg.contains("V-SHADOW-UNSUPPORTED")),
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn v5_shadow_unsupported_on_nondet_source() {
    let root = temp_project_dir("unsupported_nondet");
    prepare_project(
        &root,
        r#"observe("std.clock.now", "tier2", ctx("zone=utc"), budget(1)) -> now;
match now {
  OK => { let stable = true; }
  DEGRADED => { let stable = true; }
  INSUFFICIENT => { let stable = true; }
  DEFERRED => { let stable = true; }
}
"#,
    );

    let err =
        run_project_with_shadow_compare(&root, RunEngine::Dual, true, &ShadowOptionsV1::default())
            .expect_err("nondeterministic source must be unsupported");
    match err {
        SdkError::ShadowUnsupported(msg) => assert!(msg.contains("V-SHADOW-UNSUPPORTED")),
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn v5_shadow_forbid_commit_runtime_trace_denied() {
    let root = temp_project_dir("forbid_runtime_trace");
    prepare_project(
        &root,
        r#"observe("world.ok.signal", "tier2", ctx("x=1"), budget(3)) -> seen;
commit(seen);
let ready = true;
condition(ready);
"#,
    );

    let main = run_project_with_trace_engine_config_and_lock(
        &root,
        RunEngine::Dual,
        true,
        ExecConfig {
            step_cap: 4096,
            commit_policy: CommitPolicyMode::Normal,
        },
    )
    .expect("main trace");
    let shadow = run_project_with_trace_engine_config_and_lock(
        &root,
        RunEngine::Dual,
        true,
        ExecConfig {
            step_cap: 4096,
            commit_policy: CommitPolicyMode::ForbidCommit,
        },
    )
    .expect("shadow trace");

    let main_commit = main
        .events
        .iter()
        .find(|e| e.event == "commit_result")
        .expect("main commit_result");
    let shadow_commit = shadow
        .events
        .iter()
        .find(|e| e.event == "commit_result")
        .expect("shadow commit_result");

    assert_eq!(main_commit.allowed, Some(true));
    assert_eq!(shadow_commit.allowed, Some(false));
    assert_eq!(shadow_commit.reason.as_deref(), Some("RC-POLICY-DENIED"));
}

#[test]
fn v5_w3_reactor_io_tape_record_replay_stable_digest() {
    let root = temp_project_dir("reactor_io_tape");
    prepare_project(
        &root,
        r#"fn on_event(event) {
  let stable = true;
  condition(stable);
  return event;
}
let ready = true;
condition(ready);
"#,
    );

    let tape_path = root
        .join("target")
        .join("shadow")
        .join("reactor.io.tape.v1.txt");
    let first = run_reactor_service_with_trace_engine_and_lock(
        &root,
        &ReactorServiceOptions {
            ticks: 4,
            runtime_mode: ReactorRuntimeMode::Deterministic,
            socket_listen: None,
            runtime_report: None,
            replay_audit: None,
            io_tape_record_path: Some(tape_path.clone()),
            io_tape_replay_path: None,
            hive_caps: None,
            universe_id: None,
            domain_id: None,
        },
        RunEngine::Dual,
        true,
    )
    .expect("first trace run with tape record");

    assert!(tape_path.exists(), "io tape record must be written");
    let tape_raw = fs::read_to_string(&tape_path).expect("read io tape");
    assert!(
        !tape_raw.trim().is_empty(),
        "io tape record must not be empty"
    );

    let second = run_reactor_service_with_trace_engine_and_lock(
        &root,
        &ReactorServiceOptions {
            ticks: 4,
            runtime_mode: ReactorRuntimeMode::Deterministic,
            socket_listen: None,
            runtime_report: None,
            replay_audit: None,
            io_tape_record_path: None,
            io_tape_replay_path: Some(tape_path),
            hive_caps: None,
            universe_id: None,
            domain_id: None,
        },
        RunEngine::Dual,
        true,
    )
    .expect("second trace run with tape replay");

    assert_eq!(
        trace_required_digest(&first.events),
        trace_required_digest(&second.events)
    );
}
