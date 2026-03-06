use std::fs;

use ocl_sdk::{sign_contract_json_v19, verify_contract_json_signature_v19};
use serde_json::json;

#[path = "v19_gate_f_common.rs"]
mod f19;

#[test]
fn v19_rollback_rehearsal_restores_previous_manifest() {
    f19::ensure_run_manifest();
    let fixture = f19::ensure_release_fixture_v19();

    let release_root = fixture.release_root.clone();
    let backup_manifest = release_root.join("editor_release_manifest.prev.json");
    let backup_sig = release_root.join("editor_release_manifest.prev.json.sig");

    let original_manifest_bytes = fs::read(&fixture.manifest_path).expect("read manifest");
    let original_manifest_hash = f19::sha256_hex_bytes(&original_manifest_bytes);
    let original_sig_bytes = fs::read(&fixture.manifest_sig_path).expect("read manifest sig");

    fs::write(&backup_manifest, &original_manifest_bytes).expect("write backup manifest");
    fs::write(&backup_sig, &original_sig_bytes).expect("write backup sig");

    let mut modified = f19::read_json(&fixture.manifest_path);
    modified["channel"] = serde_json::Value::String("open_vsx".to_string());
    f19::write_json_pretty(&fixture.manifest_path, &modified);
    let _ = sign_contract_json_v19(&fixture.manifest_path, "w18-sot-root", 1)
        .expect("resign modified manifest");

    fs::copy(&backup_manifest, &fixture.manifest_path).expect("restore manifest");
    fs::copy(&backup_sig, &fixture.manifest_sig_path).expect("restore signature");

    let restored_bytes = fs::read(&fixture.manifest_path).expect("read restored manifest");
    let restored_hash = f19::sha256_hex_bytes(&restored_bytes);
    assert_eq!(
        restored_hash, original_manifest_hash,
        "rollback must restore original manifest bytes"
    );

    let verify = verify_contract_json_signature_v19(&f19::repo_root(), &fixture.manifest_path)
        .expect("verify restored signature");
    assert!(verify.signature_verified, "restored signature must verify");

    let report = json!({
        "schema": "ocl.w19.release.rollback_rehearsal_report.v1",
        "status": "PASS",
        "backup_manifest": backup_manifest.to_string_lossy().replace('\\', "/"),
        "backup_sig": backup_sig.to_string_lossy().replace('\\', "/"),
        "restored_hash": restored_hash,
        "original_hash": original_manifest_hash,
        "signature_verified": verify.signature_verified,
        "run_manifest_ref": "target/ocl/w19/meta/run_manifest.json",
        "run_manifest_sha256": f19::run_manifest_sha256()
    });
    f19::write_report("release/rollback_rehearsal_report.json", &report);
}
