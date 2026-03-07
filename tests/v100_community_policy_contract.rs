use serde_json::{json, Value as JsonValue};

#[path = "v100_gate_g_common.rs"]
mod v100g;

#[test]
fn v100_community_policy_contract() {
    v100g::ensure_run_manifest();

    let contract = v100g::community_policy();
    assert_eq!(
        contract
            .get("contract_id")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "v1.community_contribution_policy"
    );
    assert_eq!(
        contract
            .get("schema")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "ocl.business.community_contribution_policy.v1"
    );
    assert_eq!(
        contract
            .get("version")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "v1"
    );
    assert_eq!(
        contract
            .get("cla_type")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "license_grant"
    );
    let required_for = contract
        .get("cla_required_for")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect::<Vec<String>>();
    assert_eq!(
        required_for,
        vec![
            "code".to_string(),
            "docs".to_string(),
            "contracts".to_string()
        ]
    );
    assert_eq!(
        contract
            .get("inbound")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "cla"
    );
    assert_eq!(
        contract
            .get("outbound")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "AGPL-3.0-only+commercial"
    );
    assert!(contract
        .get("reject_contribution_without_cla")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false));

    let report = json!({
        "schema": "ocl.w100.business.community_policy_report.v1",
        "status": "PASS",
        "cla_type": "license_grant",
        "cla_required_for": required_for,
        "inbound": "cla",
        "outbound": "AGPL-3.0-only+commercial",
        "reject_contribution_without_cla": true,
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100g::run_manifest_sha256()
    });
    v100g::write_report("business/community_policy_report.json", &report);
}
