use serde_json::Value as JsonValue;

#[path = "v100_gate_c_common.rs"]
mod v100c;

#[test]
fn v100_win_installer_smoke() {
    let report = v100c::ensure_win_installer_smoke_report();
    assert_eq!(
        report
            .get("status")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "PASS"
    );
    assert!(
        report
            .get("installed")
            .and_then(JsonValue::as_bool)
            .unwrap_or(false),
        "installer smoke must mark installed=true"
    );
    let checks = report
        .get("post_install_checks")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(!checks.is_empty(), "post_install_checks must not be empty");
}
