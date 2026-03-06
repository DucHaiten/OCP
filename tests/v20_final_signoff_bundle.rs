#[path = "v20_gate_h_common.rs"]
mod v20h;

#[test]
fn v20_final_signoff_bundle() {
    v20h::ensure_run_manifest();
    let (bundle, go_no_go) = v20h::ensure_final_signoff_bundle();

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
    assert_eq!(
        go_no_go
            .get("signal")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(""),
        "GO",
        "final signoff must produce GO signal"
    );

    let deterministic_claim = go_no_go
        .get("deterministic_supply_chain_replay_claim")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    assert!(
        !deterministic_claim,
        "current history replay matrix includes allow_network mode; deterministic claim must stay false"
    );
}
