use ocp_sdk::verify_contract_json_signature_v20;
use serde_json::json;

#[path = "v20_gate_h_common.rs"]
mod v20h;

#[test]
fn v20_finding_schema_contract() {
    v20h::ensure_run_manifest();

    let finding_schema_path = v20h::contracts_v20_path("finding_schema.v1.json");
    let zero_open_policy_path = v20h::contracts_v20_path("zero_open_findings_policy.v1.json");

    let finding_summary =
        verify_contract_json_signature_v20(&v20h::repo_root(), &finding_schema_path)
            .expect("verify finding_schema signature");
    let policy_summary =
        verify_contract_json_signature_v20(&v20h::repo_root(), &zero_open_policy_path)
            .expect("verify zero_open_findings_policy signature");

    assert!(finding_summary.signature_verified);
    assert!(policy_summary.signature_verified);

    let schema = v20h::finding_schema();
    let required_fields = schema
        .get("required_fields")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|v| v.as_str().unwrap_or_default().to_string())
        .collect::<Vec<String>>();

    for field in [
        "finding_id",
        "component",
        "vector",
        "severity",
        "repro_steps_hash",
        "status",
        "evidence_packet_ref",
    ] {
        assert!(
            required_fields.iter().any(|item| item == field),
            "finding_schema missing required field `{field}`"
        );
    }

    let report = json!({
        "schema": "ocp.w20.signoff.finding_schema_contract_report.v1",
        "status": "PASS",
        "finding_schema_hash": finding_summary.contract_hash_sha256,
        "zero_open_policy_hash": policy_summary.contract_hash_sha256,
        "required_fields_count": required_fields.len(),
        "run_manifest_ref": "target/ocp/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20h::run_manifest_sha256()
    });
    v20h::write_report("signoff/finding_schema_contract_report.json", &report);
}
