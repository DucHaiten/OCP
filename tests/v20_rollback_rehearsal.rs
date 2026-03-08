use std::fs;

use ocp_sdk::{sign_contract_json_v20, verify_contract_json_signature_v20};
use serde_json::json;

#[path = "v20_gate_g_common.rs"]
mod v20g;

#[test]
fn v20_rollback_rehearsal() {
    v20g::ensure_run_manifest();
    let fixture = v20g::ensure_release_fixture_v20();

    let backup_manifest = fixture.release_root.join("v1_rc_manifest.prev.json");
    let backup_sig = fixture.release_root.join("v1_rc_manifest.prev.json.sig");

    let original_manifest_bytes = fs::read(&fixture.manifest_path).expect("read manifest");
    let original_manifest_hash = v20g::sha256_hex_file(&fixture.manifest_path);
    let original_sig_bytes = fs::read(&fixture.manifest_sig_path).expect("read manifest sig");

    fs::write(&backup_manifest, &original_manifest_bytes).expect("write backup manifest");
    fs::write(&backup_sig, &original_sig_bytes).expect("write backup signature");

    let mut tampered = v20g::read_json(&fixture.manifest_path);
    tampered["target_release"] = serde_json::Value::String("v1.0-rc-tampered".to_string());
    v20g::write_json_pretty(&fixture.manifest_path, &tampered);
    let _ = sign_contract_json_v20(&fixture.manifest_path, "w18-sot-root", 1)
        .expect("sign tampered manifest");

    fs::copy(&backup_manifest, &fixture.manifest_path).expect("restore manifest");
    fs::copy(&backup_sig, &fixture.manifest_sig_path).expect("restore signature");

    let restored_hash = v20g::sha256_hex_file(&fixture.manifest_path);
    assert_eq!(
        restored_hash, original_manifest_hash,
        "rollback must restore original manifest hash"
    );

    let verify = verify_contract_json_signature_v20(&v20g::repo_root(), &fixture.manifest_path)
        .expect("verify restored manifest signature");
    assert!(verify.signature_verified, "restored signature must verify");

    let report = json!({
        "schema": "ocp.w20.release.rollback_drill_report.v1",
        "status": "PASS",
        "backup_manifest": backup_manifest.to_string_lossy().replace('\\', "/"),
        "backup_signature": backup_sig.to_string_lossy().replace('\\', "/"),
        "restored_hash": restored_hash,
        "original_hash": original_manifest_hash,
        "signature_verified": verify.signature_verified,
        "run_manifest_ref": "target/ocp/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20g::run_manifest_sha256()
    });
    v20g::write_report("release/rollback_drill_report.json", &report);
}
