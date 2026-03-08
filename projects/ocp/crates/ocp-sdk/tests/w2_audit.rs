use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocp_sdk::{
    build_audit_entries, init_project, run_reactor_service_with_lock, sync_deps_lock_v1,
    verify_audit_entries, ReactorRuntimeMode, ReactorServiceOptions,
};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocp_w2_audit_{tag}_{stamp}"))
}

fn prepare_reactor_project(root: &Path) {
    init_project(root).expect("init");
    let manifest = r#"[package]
name = "w2_audit"
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
    fs::write(root.join("Ocp.toml"), manifest).expect("write manifest");
    let source = r#"module tests.w2.audit;

fn on_event(event) {
  observe("std.net.listen", "tier2", ctx("addr=127.0.0.1:19091"), budget(5)) -> listen_res;
  observe("std.net.reply", "tier2", ctx("conn=1;status=200;body=ok"), budget(5)) -> reply_res;
  commit(reply_res);
  return event;
}

let ready = true;
condition(ready);
"#;
    fs::write(root.join("src").join("main.ocp"), source).expect("write source");
    sync_deps_lock_v1(root).expect("sync lock");
}

#[test]
fn w2_audit_chain_build_and_verify() {
    let signatures = vec![
        "aaaa1111".to_string(),
        "bbbb2222".to_string(),
        "cccc3333".to_string(),
    ];
    let entries = build_audit_entries(&signatures);
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].seq, 1);
    assert_eq!(entries[0].prev_hash, "0");
    verify_audit_entries(&entries).expect("chain verify");

    let mut tampered = entries.clone();
    tampered[1].hash = "deadbeef".to_string();
    let err = verify_audit_entries(&tampered).expect_err("tampered hash must fail");
    assert!(
        err.to_string().contains("V-AUDIT-HASH"),
        "unexpected error: {err}"
    );
}

#[test]
fn w2_reactor_replay_audit_written_and_deterministic() {
    let root = temp_project_dir("replay");
    prepare_reactor_project(&root);

    let audit_a = root.join("target").join("replay_a.audit.jsonl");
    let audit_b = root.join("target").join("replay_b.audit.jsonl");

    let opts_a = ReactorServiceOptions {
        ticks: 8,
        runtime_mode: ReactorRuntimeMode::Deterministic,
        socket_listen: Some("127.0.0.1:19091".to_string()),
        runtime_report: None,
        replay_audit: Some(audit_a.clone()),
        io_tape_record_path: None,
        io_tape_replay_path: None,
        hive_caps: None,
        universe_id: None,
        domain_id: None,
    };
    let report_a = run_reactor_service_with_lock(&root, &opts_a, true).expect("run A");
    assert!(report_a.replay_audit_written);
    assert!(report_a.audit_chain_hash.is_some());

    let opts_b = ReactorServiceOptions {
        replay_audit: Some(audit_b.clone()),
        ..opts_a
    };
    let report_b = run_reactor_service_with_lock(&root, &opts_b, true).expect("run B");
    assert!(report_b.replay_audit_written);

    let text_a = fs::read_to_string(&audit_a).expect("read audit A");
    let text_b = fs::read_to_string(&audit_b).expect("read audit B");
    assert_eq!(text_a, text_b, "replay audit must be deterministic");
    assert_eq!(
        report_a.audit_chain_hash, report_b.audit_chain_hash,
        "audit chain hash must match across replay"
    );
}
