use std::fs;

use ocl_sdk::verify_contract_json_signature_v19;
use serde_json::json;

#[path = "v19_gate_f_common.rs"]
mod f19;

#[test]
fn v19_signing_trust_root_verify_for_release_manifest() {
    f19::ensure_run_manifest();
    let fixture = f19::ensure_release_fixture_v19();

    let trust_path = f19::repo_root()
        .join("contracts")
        .join("editor")
        .join("editor_signing_trust_root.v1.json");
    let trust = f19::read_json(&trust_path);
    let allowlist = trust
        .get("pubkey_allowlist")
        .and_then(serde_json::Value::as_array)
        .expect("pubkey_allowlist");
    let min_epoch = trust
        .get("trust_epoch_min")
        .and_then(serde_json::Value::as_u64)
        .expect("trust_epoch_min");
    let max_epoch = trust
        .get("trust_epoch_max")
        .and_then(serde_json::Value::as_u64)
        .expect("trust_epoch_max");
    let source_policy = trust
        .get("source_policy")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    assert_eq!(source_policy, "bundled_immutable_only");

    let sig_value: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&fixture.manifest_sig_path).expect("read signature"),
    )
    .expect("parse signature json");
    let signer = sig_value
        .get("pubkey_id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let epoch = sig_value
        .get("trust_epoch")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or_default();

    let signer_allowed = allowlist
        .iter()
        .filter_map(serde_json::Value::as_str)
        .any(|item| item == signer);
    assert!(
        signer_allowed,
        "signature signer must be in trust root allowlist"
    );
    assert!(
        epoch >= min_epoch && epoch <= max_epoch,
        "signature trust epoch must be in allowed range"
    );

    let verified = verify_contract_json_signature_v19(&f19::repo_root(), &fixture.manifest_path)
        .expect("verify release manifest signature");
    assert!(verified.signature_verified);

    let report = json!({
        "schema": "ocl.w19.security.signing_trust_root_report.v1",
        "status": "PASS",
        "trust_contract_path": trust_path.to_string_lossy().replace('\\', "/"),
        "signer": signer,
        "trust_epoch": epoch,
        "source_policy": source_policy,
        "signature_verified": verified.signature_verified,
        "run_manifest_ref": "target/ocl/w19/meta/run_manifest.json",
        "run_manifest_sha256": f19::run_manifest_sha256()
    });
    f19::write_report("security/signing_trust_root_report.json", &report);
}
