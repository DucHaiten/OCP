use serde_json::{json, Value as JsonValue};

#[path = "v100_gate_g_common.rs"]
mod v100g;

#[test]
fn v100_cla_inbound_outbound_policy() {
    v100g::ensure_run_manifest();

    let community_policy = v100g::community_policy();
    assert_eq!(
        community_policy
            .get("cla_type")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "license_grant"
    );
    assert_eq!(
        community_policy
            .get("inbound")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "cla"
    );
    assert_eq!(
        community_policy
            .get("outbound")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "AGPL-3.0-only+commercial"
    );
    let required_for = community_policy
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

    let contributing = v100g::read_text(&v100g::repo_root().join("CONTRIBUTING.md"));
    assert!(
        contributing.contains("Inbound contribution path: `cla`")
            && contributing.contains("Outbound project model: `AGPL-3.0-only+commercial`"),
        "CONTRIBUTING.md must describe inbound/outbound CLA policy"
    );

    let report = json!({
        "schema": "ocp.w100.business.cla_inbound_outbound_policy_report.v1",
        "status": "PASS",
        "cla_type": "license_grant",
        "inbound": "cla",
        "outbound": "AGPL-3.0-only+commercial",
        "cla_required_for": required_for,
        "run_manifest_ref": "target/ocp/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100g::run_manifest_sha256()
    });
    v100g::write_report("business/cla_inbound_outbound_policy_report.json", &report);
}
