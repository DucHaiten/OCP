#[path = "v18_gate_b_common.rs"]
mod common;

#[test]
fn v18_cassette_upgrade_v08_to_v17_rehearsal_writes_schema_locked_report() {
    let report = common::generate_cassette_upgrade_report();
    assert_eq!(
        report
            .get("schema")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(""),
        "ocl.w18.migration.cassette_upgrade_report.v1"
    );
    assert!(
        report
            .get("entries")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
            >= 3,
        "cassette upgrade report must include migrated entries"
    );
    assert!(
        report
            .get("unique_blocks")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
            >= 2,
        "cassette upgrade report must include dedup blocks"
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
