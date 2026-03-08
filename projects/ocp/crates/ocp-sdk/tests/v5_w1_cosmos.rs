use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocp_sdk::{
    enforce_universe_match_v1, init_cosmos_v1, init_project, resolve_universe_v1,
    sync_cosmos_lock_v1, sync_deps_lock_v1, sync_policy_lock_v1,
};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocp_v5_w1_{tag}_{stamp}"))
}

fn repo_root_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
        .join("..")
}

fn prepare_project_with_policy(root: &Path) {
    init_project(root).expect("init project");
    let manifest = r#"[package]
name = "v5_w1_cosmos"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["*"]
deny = []

[policy]
budget_profile = "ci_default"
"#;
    fs::write(root.join("Ocp.toml"), manifest).expect("write manifest");
    fs::write(
        root.join("src").join("main.ocp"),
        "let ok = true;\ncondition(ok);\n",
    )
    .expect("write source");
    sync_deps_lock_v1(root).expect("sync deps lock");
}

#[test]
fn v5_w1_policy_cosmos_lock_and_resolve_universe_pass() {
    let root = temp_project_dir("pass");
    prepare_project_with_policy(&root);
    let repo_root = repo_root_dir();
    let sign_key = repo_root.join("projects/ocp/security/dev-root-1.signing.key.toml");
    let trust_store = repo_root.join("projects/ocp/security/trust.store.toml");

    let policy = sync_policy_lock_v1(&root).expect("policy lock sync");
    assert_eq!(policy.policy_profile_id, "ci_default");
    assert!(policy.lock_path.exists(), "policy lock missing");

    let cosmos = init_cosmos_v1(&root, "ci").expect("cosmos init");
    assert_eq!(cosmos.universes_written, 2);
    assert!(cosmos.cosmos_path.exists(), "cosmos.toml missing");

    let lock = sync_cosmos_lock_v1(
        &root,
        true,
        Some("dev-root-1"),
        Some(&sign_key),
        Some(&trust_store),
    )
    .expect("cosmos lock sync");
    assert_eq!(lock.universes_synced, 2);
    assert!(lock.lock_path.exists(), "cosmos lock missing");

    let selected =
        resolve_universe_v1(&root, true, Some("ci_locked")).expect("resolve universe ci_locked");
    assert_eq!(selected.universe_id, "ci_locked");
    assert_eq!(selected.runtime_mode, "deterministic");
    assert_eq!(selected.engine, "dual");
    assert_eq!(selected.policy_profile_id, "ci_default");

    enforce_universe_match_v1(
        &selected,
        Some("deterministic"),
        Some("dual"),
        Some("ci_default"),
        true,
    )
    .expect("universe mismatch check");
}

#[test]
fn v5_w1_locked_cosmos_requires_universe_id() {
    let root = temp_project_dir("require_universe");
    prepare_project_with_policy(&root);
    let repo_root = repo_root_dir();
    let sign_key = repo_root.join("projects/ocp/security/dev-root-1.signing.key.toml");
    let trust_store = repo_root.join("projects/ocp/security/trust.store.toml");

    sync_policy_lock_v1(&root).expect("policy lock sync");
    init_cosmos_v1(&root, "ci").expect("cosmos init");
    sync_cosmos_lock_v1(
        &root,
        true,
        Some("dev-root-1"),
        Some(&sign_key),
        Some(&trust_store),
    )
    .expect("cosmos lock sync");

    let err = resolve_universe_v1(&root, true, None).expect_err("must require universe id");
    assert!(
        err.to_string().contains("V-UNIVERSE-REQUIRED"),
        "unexpected error: {err}"
    );
}

#[test]
fn v5_w1_universe_flag_rejected_without_cosmos() {
    let root = temp_project_dir("no_cosmos");
    prepare_project_with_policy(&root);

    let err = resolve_universe_v1(&root, false, Some("ci_locked"))
        .expect_err("must reject universe when cosmos missing");
    assert!(
        err.to_string().contains("V-UNIVERSE-NO-COSMOS"),
        "unexpected error: {err}"
    );
}
