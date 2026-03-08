use std::fs;

use serde_json::json;

#[path = "v19_gate_f_common.rs"]
mod f19;

#[test]
fn v19_node_supplychain_lock_policy() {
    let run_manifest = f19::ensure_run_manifest();
    let _fixture = f19::ensure_release_fixture_v19();

    let node_version = run_manifest
        .get("node_version")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let pnpm_version = run_manifest
        .get("pnpm_version")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let declared_pnpm_lock_hash = run_manifest
        .get("pnpm_lock_hash")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("missing")
        .to_string();

    assert!(
        !node_version.is_empty(),
        "node_version must be present in run manifest"
    );
    assert!(
        !pnpm_version.is_empty(),
        "pnpm_version must be present in run manifest"
    );

    let lock_path = f19::repo_root()
        .join("editor")
        .join("vscode")
        .join("ocp")
        .join("pnpm-lock.yaml");
    let (lock_present, lock_hash_match, actual_lock_hash) = if lock_path.exists() {
        let bytes = fs::read(&lock_path).expect("read pnpm lock");
        let hash = f19::sha256_hex_bytes(&bytes);
        (true, declared_pnpm_lock_hash == hash, hash)
    } else {
        (
            false,
            declared_pnpm_lock_hash == "missing",
            "missing".to_string(),
        )
    };
    assert!(
        lock_hash_match,
        "pnpm_lock_hash in run manifest must match file hash or explicit missing marker"
    );

    let report = json!({
        "schema": "ocp.w19.release.node_supplychain_report.v1",
        "status": "PASS",
        "lock_path": lock_path.to_string_lossy().replace('\\', "/"),
        "lock_present": lock_present,
        "declared_pnpm_lock_hash": declared_pnpm_lock_hash,
        "actual_pnpm_lock_hash": actual_lock_hash,
        "node_version": node_version,
        "pnpm_version": pnpm_version,
        "run_manifest_ref": "target/ocp/w19/meta/run_manifest.json",
        "run_manifest_sha256": f19::run_manifest_sha256()
    });
    f19::write_report("release/node_supplychain_report.json", &report);
}
