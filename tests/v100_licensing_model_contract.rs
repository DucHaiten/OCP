use serde_json::{json, Value as JsonValue};

#[path = "v100_gate_g_common.rs"]
mod v100g;

#[test]
fn v100_licensing_model_contract() {
    v100g::ensure_run_manifest();

    let contract = v100g::licensing_model();
    assert_eq!(
        contract
            .get("contract_id")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "v1.licensing_model"
    );
    assert_eq!(
        contract
            .get("schema")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "ocp.business.licensing_model.v1"
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
            .get("model")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "dual_license"
    );
    assert_eq!(
        contract
            .get("oss_license_id")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "AGPL-3.0-only"
    );
    assert_eq!(
        contract
            .get("oss_license_file")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "LICENSE"
    );
    assert!(contract
        .get("cla_required")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false));
    let pointer = contract
        .get("commercial_terms_pointer")
        .and_then(JsonValue::as_object)
        .unwrap_or_else(|| panic!("commercial_terms_pointer must be object"));
    assert!(pointer.contains_key("vi") && pointer.contains_key("en"));

    let report = json!({
        "schema": "ocp.w100.business.licensing_model_report.v1",
        "status": "PASS",
        "model": "dual_license",
        "oss_license_id": "AGPL-3.0-only",
        "oss_license_file": "LICENSE",
        "cla_required": true,
        "commercial_terms_pointer": {
            "vi": pointer.get("vi").and_then(JsonValue::as_str).unwrap_or(""),
            "en": pointer.get("en").and_then(JsonValue::as_str).unwrap_or("")
        },
        "run_manifest_ref": "target/ocp/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100g::run_manifest_sha256()
    });
    v100g::write_report("business/licensing_model_report.json", &report);
}
