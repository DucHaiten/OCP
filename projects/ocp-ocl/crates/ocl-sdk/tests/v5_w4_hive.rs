use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocl_sdk::{
    init_cosmos_v1, init_project, resolve_hive_caps_v1, run_reactor_service_with_lock,
    sync_cosmos_lock_v1, sync_deps_lock_v1, sync_policy_lock_v1, CosmosHiveV1, ReactorRuntimeMode,
    ReactorServiceOptions,
};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocl_v5_w4_{tag}_{stamp}"))
}

fn write_manifest(root: &Path) {
    let manifest = r#"[package]
name = "v5_w4_hive"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[runtime]
std_net = true
mailbox_max_depth = 32
io_max_events_per_tick = 4
default_deadline_ms = 250
actor_max = 1

[permissions.package]
allow = ["std.net.listen", "std.net.reply", "std.log.info"]
deny = ["std.net.poll"]
"#;
    fs::write(root.join("Ocl.toml"), manifest).expect("write manifest");
}

fn write_main(root: &Path) {
    let source = r#"module tests.v5.w4.main;

fn on_event(event) {
  observe("std.net.listen", "tier2", ctx("addr=127.0.0.1:19095"), budget(5)) -> listen_res;
  observe("std.net.reply", "tier2", ctx("conn=local;status=200;body=ok"), budget(5)) -> reply_res;
  commit(reply_res);
  return event;
}

let ready = true;
condition(ready);
"#;
    fs::write(root.join("src").join("main.ocl"), source).expect("write main");
}

fn setup_runtime_project(tag: &str) -> PathBuf {
    let root = temp_project_dir(tag);
    init_project(&root).expect("init");
    write_manifest(&root);
    write_main(&root);
    sync_deps_lock_v1(&root).expect("sync deps lock");
    root
}

#[test]
fn v5_w4_reactor_dispatch_digest_deterministic_pass() {
    let root = setup_runtime_project("dispatch_digest");
    let options = ReactorServiceOptions {
        ticks: 6,
        runtime_mode: ReactorRuntimeMode::Deterministic,
        socket_listen: Some("127.0.0.1:19095".to_string()),
        runtime_report: None,
        replay_audit: None,
        io_tape_record_path: None,
        io_tape_replay_path: None,
        hive_caps: Some(CosmosHiveV1 {
            max_swarm_workers: 4,
            max_fanout_per_task: 2,
            mailbox_max_depth: 32,
            max_spawn_per_tick: 2,
            max_domains: 8,
            max_shadow_worlds: 8,
        }),
        universe_id: None,
        domain_id: None,
    };

    let run_a = run_reactor_service_with_lock(&root, &options, true).expect("run a");
    let run_b = run_reactor_service_with_lock(&root, &options, true).expect("run b");

    assert_eq!(
        run_a.dispatch_digest256, run_b.dispatch_digest256,
        "dispatch digest must be deterministic with same input/config"
    );
    assert!(
        run_a.workers_spawned_total > 0,
        "expected workers to be spawned in W4 runtime"
    );
    assert!(
        run_a.mailbox_max_depth_observed > 0,
        "expected mailbox depth to be observed in W4 runtime"
    );
}

#[test]
fn v5_w4_locked_hive_missing_row_fail_hard() {
    let root = setup_runtime_project("hive_missing");
    init_cosmos_v1(&root, "ci").expect("init cosmos");
    sync_policy_lock_v1(&root).expect("sync policy lock");
    sync_cosmos_lock_v1(&root, false, None, None, None).expect("sync cosmos lock");

    let lock_path = root.join("cosmos.lock.v1");
    let lock_raw = fs::read_to_string(&lock_path).expect("read cosmos lock");
    let lock_without_hive = lock_raw
        .lines()
        .filter(|line| !line.trim_start().starts_with("hive="))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    fs::write(&lock_path, lock_without_hive).expect("rewrite cosmos lock without hive");

    let err = resolve_hive_caps_v1(&root, true, Some("default"))
        .expect_err("locked resolve must fail if hive row is missing");
    assert!(
        err.to_string().contains("V-HIVE-LOCK-MISSING"),
        "unexpected error: {err}"
    );
}

#[test]
fn v5_w4_locked_hive_mismatch_fail_hard() {
    let root = setup_runtime_project("hive_mismatch");
    init_cosmos_v1(&root, "ci").expect("init cosmos");
    sync_policy_lock_v1(&root).expect("sync policy lock");
    sync_cosmos_lock_v1(&root, false, None, None, None).expect("sync cosmos lock");

    let lock_path = root.join("cosmos.lock.v1");
    let lock_raw = fs::read_to_string(&lock_path).expect("read cosmos lock");
    let patched = lock_raw.replace("hive=16|8|64|4|8|8", "hive=15|8|64|4|8|8");
    fs::write(&lock_path, patched).expect("patch hive row in cosmos lock");

    let err = resolve_hive_caps_v1(&root, true, Some("default"))
        .expect_err("locked resolve must fail when hive row mismatches cosmos config");
    assert!(
        err.to_string().contains("V-HIVE-LOCK-MISMATCH"),
        "unexpected error: {err}"
    );
}
