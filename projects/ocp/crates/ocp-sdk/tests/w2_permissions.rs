use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocp_sdk::{check_project_with_lock, init_project, run_project_with_lock, sync_deps_lock_v1};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocp_w2_perm_{tag}_{stamp}"))
}

fn write_manifest(root: &Path, permissions_block: &str) {
    let manifest = format!(
        r#"[package]
name = "w2_perm"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

{permissions_block}
"#
    );
    fs::write(root.join("Ocp.toml"), manifest).expect("write manifest");
}

#[test]
fn w2_locked_requires_permissions_package() {
    let root = temp_project_dir("missing");
    init_project(&root).expect("init");
    write_manifest(&root, "");
    sync_deps_lock_v1(&root).expect("sync lock");

    let err = check_project_with_lock(&root, true).expect_err("must fail without permissions");
    assert!(
        err.to_string().contains("V-PERMISSIONS-MISSING"),
        "unexpected error: {err}"
    );
}

#[test]
fn w2_permission_deny_fail_hard() {
    let root = temp_project_dir("deny");
    init_project(&root).expect("init");
    write_manifest(
        &root,
        r#"[permissions.package]
allow = ["std.log.info"]
deny = ["std.net.reply"]
"#,
    );
    let source = r#"module tests.w2.deny;
observe("std.net.reply", "tier2", ctx("conn=1;status=200;body=x"), budget(2)) -> r;
commit(r);
"#;
    fs::write(root.join("src").join("main.ocp"), source).expect("write source");
    sync_deps_lock_v1(&root).expect("sync lock");

    let err = run_project_with_lock(&root, true).expect_err("denied key must fail hard");
    assert!(
        err.to_string().contains("V-PERMISSION-DENIED"),
        "unexpected error: {err}"
    );
}

#[test]
fn w2_module_rules_override_package() {
    let root = temp_project_dir("module_override");
    init_project(&root).expect("init");
    write_manifest(
        &root,
        r#"[permissions.package]
allow = ["std.log.info"]
deny = ["std.net.reply"]

[permissions.module.tests.w2.module_override]
allow = ["std.net.reply"]
deny = []
"#,
    );
    let source = r#"module tests.w2.module_override;
observe("std.net.reply", "tier2", ctx("conn=1;status=200;body=ok"), budget(2)) -> r;
commit(r);
let ready = true;
condition(ready);
"#;
    fs::write(root.join("src").join("main.ocp"), source).expect("write source");
    sync_deps_lock_v1(&root).expect("sync lock");

    let out = run_project_with_lock(&root, true).expect("module override should allow");
    assert!(out.steps > 0);
}
