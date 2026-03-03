use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocl_sdk::{init_project, parse_project_language_config_v071, run_project_with_lock, SdkError};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocl_manifest_{tag}_{stamp}"))
}

fn write_manifest(root: &Path, body: &str) {
    fs::write(root.join("Ocl.toml"), body).expect("write manifest");
}

fn write_main(root: &Path, body: &str) {
    fs::write(root.join("src").join("main.ocl"), body).expect("write main");
}

#[test]
fn manifest_parses_lane_and_guard_mode() {
    let manifest = r#"
[project]
lane = "locked_v06"

[language]
guard_mode = "error"
"#;
    let cfg = parse_project_language_config_v071(manifest);
    assert_eq!(cfg.lane, "locked_v06");
    assert_eq!(format!("{:?}", cfg.guard_mode), "Error");
}

#[test]
fn manifest_permission_deny_wins_and_has_hint() {
    let root = temp_project_dir("deny_wins");
    init_project(&root).expect("init");

    write_manifest(
        &root,
        r#"[package]
name = "deny_wins"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["world.*"]
deny = ["world.ok"]
"#,
    );
    write_main(
        &root,
        r#"observe("world.ok", "tier2", ctx("scene=lab"), budget(5)) -> r;
"#,
    );

    let err = run_project_with_lock(&root, false).expect_err("run must be denied by permission");
    let SdkError::PermissionDenied(msg) = err else {
        panic!("expected PermissionDenied");
    };
    assert!(msg.contains("matched deny rule=`world.ok`"));
    assert!(msg.contains("[permissions.package]"));
    assert!(msg.contains("Hint:"));
}

#[test]
fn manifest_guard_mode_error_is_enforced() {
    let root = temp_project_dir("guard_mode_error");
    init_project(&root).expect("init");

    write_manifest(
        &root,
        r#"[package]
name = "guard_mode_error"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[language]
guard_mode = "error"

[permissions.package]
allow = ["world.exists"]
"#,
    );
    write_main(
        &root,
        r#"observe("world.exists", "tier2", ctx("scene=lab"), budget(5)) -> r;
guard r;
"#,
    );

    let err = run_project_with_lock(&root, false).expect_err("guard_mode=error must fail");
    let SdkError::Runtime(runtime) = err else {
        panic!("expected Runtime error");
    };
    let msg = runtime.to_string();
    assert!(msg.contains("X-GUARD-FAILED"), "actual={msg}");
}
