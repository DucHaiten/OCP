#[path = "v100_gate_h_common.rs"]
mod v100h;

#[test]
fn v100_go_no_go_filename_contract() {
    let report = v100h::ensure_go_no_go_filename_contract_report();
    assert_eq!(
        report
            .get("status")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(""),
        "PASS"
    );
    assert_eq!(
        report
            .get("expected_filename")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(""),
        "v1_0_release_go_no_go.json"
    );
    assert!(
        report
            .get("filename_matches_contract")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        "GO/NO-GO artifact filename must stay locked"
    );
}
