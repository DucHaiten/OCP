use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocl_sdk::{
    init_project, run_reactor_service_with_lock, sync_deps_lock_v1, ReactorRuntimeMode,
    ReactorServiceOptions,
};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocl_w1_{tag}_{stamp}"))
}

fn write_w1_manifest(root: &Path) {
    let manifest = r#"[package]
name = "w1_runtime"
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

fn write_w1_main(root: &Path) {
    let source = r#"module tests.w1.main;

fn on_event(event) {
  observe("std.net.listen", "tier2", ctx("addr=127.0.0.1:19091"), budget(5)) -> listen_res;
  observe("std.net.reply", "tier2", ctx("conn=local;status=200;body=ok"), budget(5)) -> reply_res;
  commit(reply_res);
  return event;
}

let ready = true;
condition(ready);
"#;
    fs::write(root.join("src").join("main.ocl"), source).expect("write main");
}

#[test]
fn w1_runtime_modes_and_report_pass() {
    let root = temp_project_dir("modes");
    init_project(&root).expect("init");
    write_w1_manifest(&root);
    write_w1_main(&root);
    sync_deps_lock_v1(&root).expect("sync lock");

    let deterministic = run_reactor_service_with_lock(
        &root,
        &ReactorServiceOptions {
            ticks: 6,
            runtime_mode: ReactorRuntimeMode::Deterministic,
            socket_listen: Some("127.0.0.1:19091".to_string()),
            runtime_report: None,
            replay_audit: None,
        },
        true,
    )
    .expect("deterministic run");
    assert_eq!(deterministic.mode, ReactorRuntimeMode::Deterministic);
    assert_eq!(deterministic.event_count, 6);
    assert_eq!(deterministic.backpressure_count, 0);

    let report_file = root.join("target").join("runtime_report.json");
    let throughput = run_reactor_service_with_lock(
        &root,
        &ReactorServiceOptions {
            ticks: 6,
            runtime_mode: ReactorRuntimeMode::Throughput,
            socket_listen: Some("127.0.0.1:19091".to_string()),
            runtime_report: Some(report_file.clone()),
            replay_audit: None,
        },
        true,
    )
    .expect("throughput run");
    assert_eq!(throughput.mode, ReactorRuntimeMode::Throughput);
    assert!(throughput.event_count > deterministic.event_count);
    assert!(throughput.total_steps >= deterministic.total_steps);
    assert!(throughput.runtime_report_written);
    assert!(report_file.exists(), "runtime report not found");

    let report_text = fs::read_to_string(&report_file).expect("read report");
    assert!(
        report_text.contains("\"mode\":\"throughput\""),
        "missing throughput marker in report: {report_text}"
    );
}

#[test]
fn w1_runtime_socket_requires_std_net_manifest_flag() {
    let root = temp_project_dir("stdnet_flag");
    init_project(&root).expect("init");
    let manifest = r#"[package]
name = "w1_runtime_no_std_net"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["std.net.listen", "std.net.reply", "std.log.info"]
deny = ["std.net.poll"]
"#;
    fs::write(root.join("Ocl.toml"), manifest).expect("write manifest");
    write_w1_main(&root);
    sync_deps_lock_v1(&root).expect("sync lock");

    let err = run_reactor_service_with_lock(
        &root,
        &ReactorServiceOptions {
            ticks: 2,
            runtime_mode: ReactorRuntimeMode::Deterministic,
            socket_listen: Some("127.0.0.1:19091".to_string()),
            runtime_report: None,
            replay_audit: None,
        },
        true,
    )
    .expect_err("must fail when std_net runtime flag is missing");

    assert!(
        err.to_string().contains("std_net = true"),
        "unexpected error: {err}"
    );
}
