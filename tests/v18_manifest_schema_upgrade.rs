#[path = "v18_gate_b_common.rs"]
mod common;

#[test]
fn v18_manifest_schema_upgrade_rehearsal_noop_is_deterministic() {
    let report = common::generate_manifest_upgrade_report();
    assert_eq!(
        report
            .get("schema")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(""),
        "ocp.w18.migration.manifest_upgrade_report.v1"
    );
    assert_eq!(
        report.get("no_op").and_then(serde_json::Value::as_bool),
        Some(true),
        "manifest rehearsal should produce no-op proof when schema already current"
    );
    assert_eq!(
        report.get("input_hash_before"),
        report.get("input_hash_after"),
        "no-op report must keep input hash unchanged"
    );
    for key in [
        "no_op",
        "input_hash_before",
        "input_hash_after",
        "sot_ref",
        "run_manifest_ref",
    ] {
        assert!(report.get(key).is_some(), "missing required field `{key}`");
    }
}
