use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocp_runtime_core::RunEngine;
use ocp_sdk::{
    build_ocppkg_with_lock, fetch_artifact, init_project, publish_artifact, run_artifact,
    sync_deps_lock_v1, sync_deps_lock_v2, verify_supply_artifact,
};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocp_w4_{tag}_{stamp}"))
}

fn prepare_project(root: &Path) {
    init_project(root).expect("init");
    let manifest = r#"[package]
name = "w4_supply_demo"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"
http = "1.2.3"

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
}

#[test]
fn w4_build_verify_publish_fetch_run_artifact_pass() {
    let root = temp_project_dir("flow");
    prepare_project(&root);
    sync_deps_lock_v1(&root).expect("sync v1");
    sync_deps_lock_v2(&root).expect("sync v2");

    let build = build_ocppkg_with_lock(&root, true).expect("build ocppkg");
    assert!(build.artifact_path.exists(), "missing artifact");
    assert!(!build.payload_hash_blake3.is_empty());

    let verify = verify_supply_artifact(&build.artifact_path).expect("verify artifact");
    assert!(verify.valid);
    assert_eq!(verify.package_name, "w4_supply_demo");

    let registry = root.join("registry");
    let publish = publish_artifact(&build.artifact_path, &registry).expect("publish");
    assert!(publish.registry_index.exists(), "missing registry index");
    assert_eq!(publish.package_name, "w4_supply_demo");

    let out = root.join("fetched");
    let fetch = fetch_artifact("w4_supply_demo", &registry, &out).expect("fetch");
    assert!(fetch.artifact_path.exists(), "missing fetched artifact");

    let run = run_artifact(&fetch.artifact_path, RunEngine::Dual, 4096).expect("run artifact");
    assert!(run.steps > 0);
}

#[test]
fn w4_locked_fails_when_dep_signature_missing() {
    let root = temp_project_dir("sig_missing");
    prepare_project(&root);
    sync_deps_lock_v1(&root).expect("sync v1");

    let v2 = root.join("deps.lock.v2");
    let raw = fs::read_to_string(&v2).expect("read v2");
    let mut lines = Vec::new();
    for line in raw.lines() {
        if let Some(payload) = line.strip_prefix("dep=http|") {
            let mut parts: Vec<&str> = payload.split('|').collect();
            if parts.len() == 4 {
                parts[2] = "";
            }
            lines.push(format!("dep=http|{}", parts.join("|")));
        } else {
            lines.push(line.to_string());
        }
    }
    let mutated = lines.join("\n") + "\n";
    fs::write(&v2, mutated).expect("write mutated v2");

    let err = build_ocppkg_with_lock(&root, true).expect_err("locked build should fail");
    let msg = err.to_string();
    assert!(
        msg.contains("signature") || msg.contains("Supply"),
        "unexpected error message: {msg}"
    );
}
