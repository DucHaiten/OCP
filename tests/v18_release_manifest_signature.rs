use ocp_sdk::{verify_contract_json_signature_v18, W18_HASHER_VERSION};
use serde_json::json;

#[path = "v18_gate_f_common.rs"]
mod common;

#[test]
fn v18_release_manifest_signature_verifies_with_trust_store() {
    let (manifest_path, canonical_sig_path) = common::ensure_release_artifact_manifest_signed();
    let summary = verify_contract_json_signature_v18(&common::repo_root(), &manifest_path)
        .expect("verify release artifact manifest signature");
    assert!(summary.signature_verified);
    assert_eq!(summary.contract_id, "v1.release_artifact_manifest");
    assert_eq!(summary.version, "v1");
    assert_eq!(summary.signer_id.as_deref(), Some("w18-sot-root"));
    assert_eq!(summary.trust_epoch, Some(1));

    let report = json!({
        "schema": "ocp.w18.rc.release_manifest_signature_report.v1",
        "run_manifest_ref": "target/ocp/w18/meta/run_manifest.json",
        "manifest_path": manifest_path.to_string_lossy().replace('\\', "/"),
        "canonical_signature_path": canonical_sig_path.to_string_lossy().replace('\\', "/"),
        "compat_signature_path": common::w18_rc_dir().join("release_artifact_manifest.sig").to_string_lossy().replace('\\', "/"),
        "hasher_version": W18_HASHER_VERSION,
        "signature_verified": summary.signature_verified,
        "signer_id": summary.signer_id,
        "trust_epoch": summary.trust_epoch
    });
    common::write_json_pretty(
        &common::w18_rc_dir().join("release_manifest_signature_report.json"),
        &report,
    );
}
