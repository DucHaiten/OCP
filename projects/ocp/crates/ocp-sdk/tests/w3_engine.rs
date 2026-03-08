use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use ocp_runtime_core::RunEngine;
use ocp_sdk::{init_project, run_project_with_engine_and_lock, sync_deps_lock_v1};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocp_sdk_w3_{tag}_{stamp}"))
}

fn prepare_project(root: &Path) {
    init_project(root).expect("init");
    let manifest = r#"[package]
name = "w3_sdk_engine"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["*"]
deny = ["std.net.poll"]
"#;
    fs::write(root.join("Ocp.toml"), manifest).expect("write manifest");
    fs::write(
        root.join("src").join("main.ocp"),
        "let ready = true;\ncondition(ready);\n",
    )
    .expect("write main");
    sync_deps_lock_v1(root).expect("sync lock");
}

#[test]
fn w3_sdk_run_project_all_engines_pass() {
    let root = temp_project_dir("all_engines");
    prepare_project(&root);

    let out_interp =
        run_project_with_engine_and_lock(&root, RunEngine::Interpreter, true).expect("interp");
    let out_bytecode =
        run_project_with_engine_and_lock(&root, RunEngine::Bytecode, true).expect("bytecode");
    let out_dual = run_project_with_engine_and_lock(&root, RunEngine::Dual, true).expect("dual");

    assert!(out_interp.steps > 0);
    assert_eq!(out_interp.steps, out_bytecode.steps);
    assert_eq!(out_interp.steps, out_dual.steps);
}
