#[path = "v100_gate_h_common.rs"]
mod v100h;

#[test]
fn v100_release_publish_rehearsal() {
    let report = v100h::ensure_publish_rehearsal_report();
    assert_eq!(
        report
            .get("status")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(""),
        "PASS"
    );
    assert_eq!(
        report
            .get("mode")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(""),
        "draft_staging_only"
    );
    assert!(
        !report
            .get("production_publish_executed")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true),
        "gate H rehearsal must not publish production artifacts"
    );
    assert!(
        report
            .get("staging_required_assets_present")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        "staging rehearsal assets must all be present"
    );
}
