use ocp_sdk::verify_sot_alias_contract_v18;
use serde_json::json;

#[path = "v100_gate_a_common.rs"]
mod v100;

#[test]
fn v100_sot_namespace_alias_contract() {
    v100::ensure_run_manifest();

    let summary = verify_sot_alias_contract_v18(&v100::repo_root()).expect("verify alias contract");
    assert!(
        summary
            .items
            .iter()
            .all(|item| !item.mirror_exists || item.bytes_identical),
        "mirror files must be byte-identical when present"
    );
    let has_release_required_alias = summary.items.iter().any(|item| {
        item.canonical_path == "contracts/release/v1.0/release_required_contracts.v1.json"
            && item.mirror_path == "contracts/v1/release_required_contracts.v1.json"
    });
    assert!(
        has_release_required_alias,
        "missing alias pair for release_required_contracts.v1.json"
    );

    let report = json!({
        "schema": "ocp.w100.sot_namespace_alias_report.v1",
        "status": "PASS",
        "alias_count": summary.alias_count,
        "verified_count": summary.verified_count,
        "items": summary.items.iter().map(|item| json!({
            "canonical_path": item.canonical_path,
            "mirror_path": item.mirror_path,
            "mirror_exists": item.mirror_exists,
            "bytes_identical": item.bytes_identical
        })).collect::<Vec<_>>(),
        "run_manifest_ref": "target/ocp/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100::run_manifest_sha256()
    });
    v100::write_report("contracts/sot_namespace_alias_report.json", &report);
}
