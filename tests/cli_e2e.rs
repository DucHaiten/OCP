use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocl_sdk::{init_project, run_project_with_lock, sync_deps_lock_v1, SdkError};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocl_cli_e2e_{tag}_{stamp}"))
}

fn write_manifest(root: &Path, body: &str) {
    fs::write(root.join("Ocl.toml"), body).expect("write manifest");
}

fn write_main(root: &Path, body: &str) {
    fs::write(root.join("src").join("main.ocl"), body).expect("write main");
}

#[test]
fn cli_e2e_guard_mode_return_is_non_breaking() {
    let root = temp_project_dir("guard_return");
    init_project(&root).expect("init");

    write_manifest(
        &root,
        r#"[package]
name = "guard_return"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[language]
guard_mode = "return"

[permissions.package]
allow = ["world.exists"]
"#,
    );
    write_main(
        &root,
        r#"observe("world.exists", "tier2", ctx("scene=lab"), budget(5)) -> r;
guard r;
condition(true);
"#,
    );

    let summary = run_project_with_lock(&root, false).expect("guard_mode=return should pass");
    assert!(summary.steps > 0);
}

#[test]
fn cli_e2e_locked_requires_permissions_package() {
    let root = temp_project_dir("locked_requires_permissions");
    init_project(&root).expect("init");

    write_manifest(
        &root,
        r#"[package]
name = "locked_requires_permissions"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"
"#,
    );
    write_main(&root, "condition(true);\n");
    sync_deps_lock_v1(&root).expect("sync deps lock");

    let err = run_project_with_lock(&root, true).expect_err("locked run must require permissions");
    let SdkError::PermissionMissing(msg) = err else {
        panic!("expected PermissionMissing");
    };
    assert!(msg.contains("V-PERMISSIONS-MISSING"), "actual={msg}");
}
