use serde_json::json;

#[path = "v20_gate_a_common.rs"]
mod v20;

#[test]
fn v20_threat_model_contract() {
    v20::ensure_run_manifest();

    let path = v20::contracts_root()
        .join("v20")
        .join("threat_model.v1.json");
    let value = v20::read_json(&path);

    assert_eq!(
        value.get("contract_id").and_then(serde_json::Value::as_str),
        Some("v20.threat_model"),
        "threat_model contract_id must be stable"
    );
    let threat_model_id = value
        .get("threat_model_id")
        .and_then(serde_json::Value::as_str)
        .expect("threat_model_id");
    assert!(
        !threat_model_id.trim().is_empty(),
        "threat_model_id must not be empty"
    );

    let assets = value
        .get("scope")
        .and_then(|v| v.get("assets"))
        .and_then(serde_json::Value::as_array)
        .expect("scope.assets");
    assert!(!assets.is_empty(), "scope.assets must not be empty");

    let trust_boundaries = value
        .get("scope")
        .and_then(|v| v.get("trust_boundaries"))
        .and_then(serde_json::Value::as_array)
        .expect("scope.trust_boundaries");
    assert!(
        !trust_boundaries.is_empty(),
        "scope.trust_boundaries must not be empty"
    );

    let rubric = value
        .get("severity_rubric")
        .and_then(serde_json::Value::as_object)
        .expect("severity_rubric");
    for key in ["CRITICAL", "HIGH", "MEDIUM", "LOW"] {
        assert!(
            rubric.contains_key(key),
            "severity_rubric missing required key `{key}`"
        );
    }

    let report = json!({
        "schema": "ocp.w20.threat_model_report.v1",
        "status": "PASS",
        "threat_model_id": threat_model_id,
        "assets_count": assets.len(),
        "trust_boundaries_count": trust_boundaries.len(),
        "run_manifest_ref": "target/ocp/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20::run_manifest_sha256()
    });
    v20::write_report("contracts/threat_model_report.json", &report);
}
