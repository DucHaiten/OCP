use ocl_sdk::verify_contract_json_signature_v19;
use serde_json::json;

#[path = "v19_gate_f_common.rs"]
mod f19;

#[test]
fn v19_vsix_signature_verify_chain() {
    f19::ensure_run_manifest();
    let fixture = f19::ensure_release_fixture_v19();

    let summary = verify_contract_json_signature_v19(&f19::repo_root(), &fixture.manifest_path)
        .expect("release manifest signature verify");
    assert!(summary.signature_verified);
    assert_eq!(summary.signer_id.as_deref(), Some("w18-sot-root"));
    assert_eq!(summary.trust_epoch, Some(1));

    let report = json!({
        "schema": "ocl.w19.release.vsix_verify_report.v1",
        "status": "PASS",
        "contract_id": summary.contract_id,
        "manifest_hash": summary.contract_hash_sha256,
        "signer_id": summary.signer_id,
        "trust_epoch": summary.trust_epoch,
        "run_manifest_ref": "target/ocl/w19/meta/run_manifest.json",
        "run_manifest_sha256": f19::run_manifest_sha256()
    });
    f19::write_report("release/vsix_verify_report.json", &report);
}
