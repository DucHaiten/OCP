#[path = "v100_gate_h_common.rs"]
mod v100h;

#[test]
fn v100_release_go_no_go() {
    let (bundle, go_no_go) = v100h::ensure_final_signoff_bundle();
    assert_eq!(
        go_no_go
            .get("status")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(""),
        "PASS"
    );

    let bundle_clean = bundle
        .get("status")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        == "PASS";
    let prerequisites_done = go_no_go
        .get("prerequisite_gates_all_done")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let blocking_open = go_no_go
        .get("release_blocking_open_count")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(999);
    let expected = if bundle_clean && prerequisites_done && blocking_open == 0 {
        "GO"
    } else {
        "NO_GO"
    };
    assert_eq!(
        go_no_go
            .get("signal")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(""),
        expected,
        "GO/NO-GO signal must reflect prereq gates + findings + signoff bundle cleanliness"
    );
}
