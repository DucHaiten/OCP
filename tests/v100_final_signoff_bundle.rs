#[path = "v100_gate_h_common.rs"]
mod v100h;

#[test]
fn v100_final_signoff_bundle() {
    let (bundle, go_no_go) = v100h::ensure_final_signoff_bundle();
    assert_eq!(
        bundle
            .get("status")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(""),
        "PASS"
    );
    assert_eq!(
        go_no_go
            .get("status")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(""),
        "PASS"
    );
    let missing = bundle
        .get("missing_required_artifacts")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(
        missing.is_empty(),
        "final signoff bundle still misses required artifacts"
    );
}
