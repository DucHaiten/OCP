use serde_json::Value as JsonValue;

#[path = "v100_gate_c_common.rs"]
mod v100c;

#[test]
fn v100_portable_install_smoke() {
    let report = v100c::ensure_portable_install_report();
    assert_eq!(
        report.get("status").and_then(JsonValue::as_str).unwrap_or(""),
        "PASS"
    );
    let assets = report
        .get("portable_assets")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(!assets.is_empty(), "portable_assets must not be empty");
    let extracted = report
        .get("extracted_targets")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    assert_eq!(
        assets.len(),
        extracted.len(),
        "every portable asset must have extracted target report"
    );
}
