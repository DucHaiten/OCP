use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocl_sdk::{init_project, run_project_with_lock, SdkError};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocl_manifest_v72_{tag}_{stamp}"))
}

fn write_manifest(root: &Path, body: &str) {
    fs::write(root.join("Ocl.toml"), body).expect("write manifest");
}

fn write_main(root: &Path, body: &str) {
    fs::write(root.join("src").join("main.ocl"), body).expect("write main");
}

fn expect_permission_denied(err: SdkError) -> String {
    let SdkError::PermissionDenied(msg) = err else {
        panic!("expected PermissionDenied, got {err}");
    };
    msg
}

#[test]
fn manifest_fs_missing_block_denies_with_rc_and_hint() {
    let root = temp_project_dir("fs_missing");
    init_project(&root).expect("init");

    write_manifest(
        &root,
        r#"[package]
name = "fs_missing"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["std.fs.*"]
"#,
    );
    write_main(
        &root,
        r#"observe("std.fs.read_text", "tier2", ctx("path=./data/sample.txt"), budget(5)) -> r;
"#,
    );

    let msg = expect_permission_denied(
        run_project_with_lock(&root, false)
            .expect_err("std.fs must be denied without std_fs block"),
    );
    assert!(msg.contains("RC-FS-PERMISSION-DENIED"), "actual={msg}");
    assert!(msg.contains("[permissions.std_fs]"), "actual={msg}");
    assert!(msg.contains("read = ["), "actual={msg}");
}

#[test]
fn manifest_fs_present_allows_read_key() {
    let root = temp_project_dir("fs_allowed");
    init_project(&root).expect("init");

    write_manifest(
        &root,
        r#"[package]
name = "fs_allowed"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["std.fs.*"]

[permissions.std_fs]
read = ["./data/**"]
"#,
    );
    write_main(
        &root,
        r#"observe("std.fs.read_text", "tier2", ctx("path=./data/sample.txt"), budget(5)) -> r;
"#,
    );

    let summary = run_project_with_lock(&root, false).expect("std.fs read should be allowed");
    assert!(summary.steps >= 1);
}

#[test]
fn manifest_package_deny_overrides_std_fs_allow() {
    let root = temp_project_dir("deny_override");
    init_project(&root).expect("init");

    write_manifest(
        &root,
        r#"[package]
name = "deny_override"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["std.fs.*"]
deny = ["std.fs.read_text"]

[permissions.std_fs]
read = ["./data/**"]
"#,
    );
    write_main(
        &root,
        r#"observe("std.fs.read_text", "tier2", ctx("path=./data/sample.txt"), budget(5)) -> r;
"#,
    );

    let msg = expect_permission_denied(
        run_project_with_lock(&root, false).expect_err("deny must override std_fs allow"),
    );
    assert!(
        msg.contains("matched deny rule=`std.fs.read_text`"),
        "actual={msg}"
    );
    assert!(msg.contains("[permissions.package]"), "actual={msg}");
}

#[test]
fn manifest_kv_requires_enabled_true() {
    let root = temp_project_dir("kv_disabled");
    init_project(&root).expect("init");

    write_manifest(
        &root,
        r#"[package]
name = "kv_disabled"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["std.kv.*"]

[permissions.std_kv]
enabled = false
max_keys = 128
"#,
    );
    write_main(
        &root,
        r#"observe("std.kv.get", "tier2", ctx("key=app.flag"), budget(5)) -> r;
"#,
    );

    let msg =
        expect_permission_denied(run_project_with_lock(&root, false).expect_err("kv disabled"));
    assert!(msg.contains("RC-KV-PERMISSION-DENIED"), "actual={msg}");
    assert!(msg.contains("enabled = true"), "actual={msg}");
}

#[test]
fn manifest_time_requires_enabled_true() {
    let root = temp_project_dir("time_disabled");
    init_project(&root).expect("init");

    write_manifest(
        &root,
        r#"[package]
name = "time_disabled"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["std.time.*"]

[permissions.std_time]
enabled = false
tick_mode = "logical"
dt_ms = 16
"#,
    );
    write_main(
        &root,
        r#"observe("std.time.tick_info", "tier2", ctx("scope=clock"), budget(5)) -> r;
"#,
    );

    let msg =
        expect_permission_denied(run_project_with_lock(&root, false).expect_err("time disabled"));
    assert!(msg.contains("RC-TIME-DISABLED"), "actual={msg}");
    assert!(msg.contains("enabled = true"), "actual={msg}");
}
