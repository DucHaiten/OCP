#[path = "v18_gate_b_common.rs"]
mod common;

#[test]
fn v18_pack_abi_upgrade_rehearsal_noop_keeps_contract_hash() {
    let report = common::generate_pack_abi_upgrade_report();
    assert_eq!(
        report
            .get("schema")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(""),
        "ocp.w18.migration.pack_abi_upgrade_report.v1"
    );
    assert_eq!(
        report.get("no_op").and_then(serde_json::Value::as_bool),
        Some(true),
        "pack abi rehearsal should be no-op when ABI version unchanged"
    );
    assert_eq!(
        report.get("input_hash_before"),
        report.get("input_hash_after"),
        "no-op pack abi report must keep input hash unchanged"
    );
    assert_eq!(
        report
            .get("supports_wasi_v1")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        report
            .get("supports_native_cap_v1")
            .and_then(serde_json::Value::as_bool),
        Some(true)
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
