#[path = "w17_gate_g_common.rs"]
mod w17;

use std::path::PathBuf;

use ocp_sdk::{verify_w17_contract_set_v17, W17_REQUIRED_CONTRACT_FILES};
use serde_json::{json, Value as JsonValue};

#[test]
fn v017_release_guard_verifies_risk_locks_and_contract_sot() {
    let contracts_root = PathBuf::from("contracts").join("w17");
    let set_summary = verify_w17_contract_set_v17(&contracts_root).expect("verify w17 contracts");
    assert_eq!(
        set_summary.required_count,
        W17_REQUIRED_CONTRACT_FILES.len()
    );
    assert_eq!(
        set_summary.verified_count,
        W17_REQUIRED_CONTRACT_FILES.len()
    );

    let policy_path = contracts_root.join("risk_lock_policies.v1.json");
    let policy_raw = w17::read_utf8(&policy_path);
    let policy: JsonValue = serde_json::from_str(&policy_raw).expect("parse risk_lock_policies");

    let lane_literals = policy
        .get("lane_literals")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(ToOwned::to_owned))
        .collect::<Vec<String>>();
    assert_eq!(
        lane_literals,
        vec![
            "locked_v071".to_string(),
            "locked_v06".to_string(),
            "quarantine".to_string()
        ]
    );

    let risk_locks = policy
        .get("risk_locks")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    assert_eq!(risk_locks.len(), 17, "RL-1..RL-17 must be present");
    for item in &risk_locks {
        let status = item
            .get("status")
            .and_then(JsonValue::as_str)
            .unwrap_or("-");
        assert_eq!(status, "locked", "all risk locks must remain locked");
    }

    let report = json!({
        "schema": "ocp.w17.release_guard_report.v1",
        "run_manifest_ref": "target/ocp/w17/meta/run_manifest.json",
        "contract_set": {
            "required_count": set_summary.required_count,
            "verified_count": set_summary.verified_count
        },
        "risk_lock_policy": {
            "lane_literals": lane_literals,
            "risk_lock_count": risk_locks.len(),
            "all_locked": true
        },
        "pass": true
    });
    w17::write_rc_report("v017_release_guard_report.json", &report);
}
