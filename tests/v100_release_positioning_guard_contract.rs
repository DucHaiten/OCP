use serde_json::{json, Value as JsonValue};

#[path = "v100_gate_a_common.rs"]
mod v100;

#[test]
fn v100_release_positioning_guard_contract() {
    v100::ensure_run_manifest();

    let guard = v100::read_json(&v100::release_positioning_guard_path());
    assert_eq!(
        guard
            .get("contract_id")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "v1.release_positioning_guard"
    );
    assert_eq!(
        guard
            .get("supported_profile_claim")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "supported_profile_only"
    );
    assert_eq!(
        guard
            .get("performance_positioning")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "governed_runtime_not_compute_kernel_replacement"
    );
    let deterministic_claim = guard
        .get("deterministic_supply_chain_replay_claim")
        .and_then(JsonValue::as_bool)
        .unwrap_or(true);
    assert!(
        !deterministic_claim,
        "deterministic supply-chain replay full claim must stay false"
    );

    let forbidden = guard
        .get("forbidden_claims")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default()
        .iter()
        .filter_map(JsonValue::as_str)
        .map(|v| v.to_string())
        .collect::<Vec<String>>();
    assert!(
        forbidden
            .iter()
            .any(|item| item == "deterministic_supply_chain_replay_full"),
        "forbidden claims must include deterministic_supply_chain_replay_full"
    );

    let report = json!({
        "schema": "ocp.w100.release_positioning_guard_report.v1",
        "status": "PASS",
        "deterministic_supply_chain_replay_claim": deterministic_claim,
        "forbidden_claims": forbidden,
        "run_manifest_ref": "target/ocp/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100::run_manifest_sha256()
    });
    v100::write_report("contracts/release_positioning_guard_report.json", &report);
}
