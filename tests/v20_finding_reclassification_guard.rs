#[path = "v20_gate_h_common.rs"]
mod v20h;

#[test]
fn v20_finding_reclassification_guard() {
    v20h::ensure_run_manifest();
    let report = v20h::ensure_finding_reclassification_report();
    assert_eq!(
        report
            .get("status")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(""),
        "PASS"
    );

    let attempts = report
        .get("attempts")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(
        attempts.iter().any(|item| {
            item.get("allowed").and_then(serde_json::Value::as_bool) == Some(false)
                && item.get("reason_code").and_then(serde_json::Value::as_str)
                    == Some("RC-RECLASS-EVIDENCE-REQUIRED")
        }),
        "anti-gaming block for missing evidence must exist"
    );
}
