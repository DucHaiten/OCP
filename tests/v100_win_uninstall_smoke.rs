use serde_json::Value as JsonValue;

#[path = "v100_gate_c_common.rs"]
mod v100c;

#[test]
fn v100_win_uninstall_smoke() {
    let report = v100c::ensure_win_uninstall_smoke_report();
    assert_eq!(
        report
            .get("status")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "PASS"
    );
    assert!(
        report
            .get("uninstalled")
            .and_then(JsonValue::as_bool)
            .unwrap_or(false),
        "uninstall smoke must mark uninstalled=true"
    );
    let stale = report
        .get("stale_processes_after_uninstall")
        .and_then(JsonValue::as_u64)
        .unwrap_or(1);
    assert_eq!(stale, 0, "stale processes after uninstall must be zero");
}
