use serde_json::{json, Value as JsonValue};

#[path = "v100_gate_g_common.rs"]
mod v100g;

#[test]
fn v100_licensing_model_decision_lock() {
    v100g::ensure_run_manifest();

    let contract = v100g::licensing_model();
    let model = contract.get("model").unwrap_or(&JsonValue::Null);
    assert!(model.is_string(), "model must be a single locked string");
    assert_eq!(model.as_str().unwrap_or(""), "dual_license");
    assert!(
        contract
            .get("cla_required")
            .and_then(JsonValue::as_bool)
            .unwrap_or(false)
    );
    assert!(
        contract.get("model_options").is_none()
            && contract.get("allowed_models").is_none()
            && contract.get("options").is_none(),
        "licensing model contract must not expose unresolved option lists"
    );

    let report = json!({
        "schema": "ocl.w100.business.licensing_model_decision_report.v1",
        "status": "PASS",
        "locked_model": "dual_license",
        "cla_required": true,
        "ambiguous_option_fields_present": false,
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100g::run_manifest_sha256()
    });
    v100g::write_report("business/licensing_model_decision_report.json", &report);
}
