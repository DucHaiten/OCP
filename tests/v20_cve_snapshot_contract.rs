use ocp_sdk::verify_contract_json_signature_v20;
use serde_json::json;

#[path = "v20_gate_f_common.rs"]
mod v20f;

#[test]
fn v20_cve_snapshot_contract() {
    v20f::ensure_run_manifest();
    let repo_root = v20f::repo_root();
    let contract_path = v20f::contracts_root()
        .join("v20")
        .join("cve_snapshot.v1.json");

    let summary = verify_contract_json_signature_v20(&repo_root, &contract_path)
        .expect("verify cve snapshot signature");
    assert!(
        summary.signature_verified,
        "cve snapshot signature must verify"
    );

    let snapshot = v20f::cve_snapshot();
    assert_eq!(
        snapshot
            .get("schema")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(""),
        "ocp.v20.cve_snapshot.v1",
        "unexpected cve snapshot schema"
    );
    assert_eq!(
        snapshot
            .get("contract_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(""),
        "v20.cve_snapshot",
        "unexpected cve snapshot contract_id"
    );

    let deny_severities = v20f::string_array_at_path(&snapshot, &["policy", "deny_severities"]);
    let deny_packages = snapshot
        .get("deny_packages")
        .and_then(serde_json::Value::as_array)
        .map(|items| items.len())
        .unwrap_or(0usize);
    let allow_exceptions = snapshot
        .get("allow_exceptions")
        .and_then(serde_json::Value::as_array)
        .map(|items| items.len())
        .unwrap_or(0usize);

    let report = json!({
        "schema": "ocp.w20.security.cve_snapshot_report.v1",
        "status": "PASS",
        "contract_path": contract_path.to_string_lossy().replace('\\', "/"),
        "signature_path": summary.signature_path.to_string_lossy().replace('\\', "/"),
        "contract_hash_sha256": summary.contract_hash_sha256,
        "signer_id": summary.signer_id,
        "trust_epoch": summary.trust_epoch,
        "deny_severities": deny_severities,
        "deny_packages_count": deny_packages,
        "allow_exceptions_count": allow_exceptions,
        "run_manifest_ref": "target/ocp/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20f::run_manifest_sha256()
    });
    v20f::write_report("security/cve_snapshot_report.json", &report);
}
