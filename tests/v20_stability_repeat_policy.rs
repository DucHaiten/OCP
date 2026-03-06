#[path = "v20_gate_h_common.rs"]
mod v20h;

#[test]
fn v20_stability_repeat_policy() {
    v20h::ensure_run_manifest();
    let report = v20h::ensure_stability_repeat_report();
    assert_eq!(
        report
            .get("status")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(""),
        "PASS"
    );

    let suite_results = report
        .get("suite_results")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(
        !suite_results.is_empty(),
        "stability_repeat_report must contain suite results"
    );
    for suite in &suite_results {
        let mismatch = suite
            .get("mismatch_count")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(999);
        assert_eq!(mismatch, 0, "repeat mismatch detected: {suite}");
    }
}
