#[path = "v100_gate_h_common.rs"]
mod v100h;

#[test]
fn v100_zero_open_findings() {
    let report = v100h::ensure_final_findings_report();
    assert_eq!(
        report
            .get("status")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(""),
        "PASS"
    );

    let groups = report
        .get("severity_groups")
        .and_then(serde_json::Value::as_object)
        .cloned()
        .unwrap_or_default();
    for level in ["CRITICAL", "HIGH", "MEDIUM", "LOW"] {
        assert!(
            groups.contains_key(level),
            "missing severity group `{level}` in final findings report"
        );
    }
    let open_critical = groups
        .get("CRITICAL")
        .and_then(|value| value.get("open"))
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(999);
    let open_high = groups
        .get("HIGH")
        .and_then(|value| value.get("open"))
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(999);
    assert_eq!(open_critical, 0, "open CRITICAL findings must be zero");
    assert_eq!(open_high, 0, "open HIGH findings must be zero");
}
