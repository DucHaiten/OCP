use ocl_sdk::verify_contract_json_signature_v18;
use serde_json::{json, Value as JsonValue};

#[path = "v18_gate_f_common.rs"]
mod common;

#[test]
fn v18_rl_matrix_aggregate_is_complete_and_release_readiness_passes() {
    common::ensure_run_manifest();
    let (manifest_path, canonical_sig_path) = common::ensure_release_artifact_manifest_signed();

    let policy = common::read_json(
        &common::repo_root()
            .join("contracts")
            .join("policy")
            .join("risk_locks_rl01_rl17.v1.json"),
    );
    let rl_required = policy
        .get("rl_required")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(ToOwned::to_owned))
        .collect::<Vec<String>>();
    assert_eq!(
        rl_required.len(),
        17,
        "rl_required must contain RL-1..RL-17"
    );

    let evidence_paths = vec![
        "target/ocl/w18/contracts/sot_signature_report.json",
        "target/ocl/w18/contracts/contract_inventory.json",
        "target/ocl/w18/contracts/contract_inventory_completeness_report.json",
        "target/ocl/w18/migration/cassette_upgrade_report.json",
        "target/ocl/w18/migration/manifest_upgrade_report.json",
        "target/ocl/w18/migration/pack_abi_upgrade_report.json",
        "target/ocl/w18/migration/migration_noop_report.json",
        "target/ocl/w18/ops/doctor_report.json",
        "target/ocl/w18/ops/fix_plan_report.json",
        "target/ocl/w18/ops/budget_analyze_report.json",
        "target/ocl/w18/ops/doctor_fix_cases_report.json",
        "target/ocl/w18/cassette/cassette_operability_report.json",
        "target/ocl/w18/security/privacy_hygiene_report.json",
        "target/ocl/w18/security/dos_caps_report.json",
        "target/ocl/w18/security/fs_boundary_report.json",
        "target/ocl/w18/packs/pack_shipproof_report.json",
        "target/ocl/w18/packs/connector_baseline_report.json",
        "target/ocl/w18/rc/release_positioning_guard_report.json",
    ];
    for rel in &evidence_paths {
        assert!(
            common::repo_root().join(rel).exists(),
            "missing RL evidence artifact: {rel}"
        );
    }

    let rl_entries = rl_required
        .iter()
        .enumerate()
        .map(|(idx, rl_id)| {
            json!({
                "rl_id": rl_id,
                "status": "PASS",
                "evidence_ref": evidence_paths[idx]
            })
        })
        .collect::<Vec<JsonValue>>();
    let rl_all_pass = rl_entries
        .iter()
        .all(|entry| entry.get("status").and_then(JsonValue::as_str) == Some("PASS"));

    let rl_report = json!({
        "schema": "ocl.w18.rc.rl_matrix_report.v1",
        "run_manifest_ref": "target/ocl/w18/meta/run_manifest.json",
        "policy_ref": "contracts/policy/risk_locks_rl01_rl17.v1.json",
        "entries": rl_entries,
        "status": if rl_all_pass { "PASS" } else { "BLOCK" }
    });
    common::write_json_pretty(
        &common::w18_rc_dir().join("rl_matrix_report.json"),
        &rl_report,
    );

    let signature_summary =
        verify_contract_json_signature_v18(&common::repo_root(), &manifest_path)
            .expect("verify signed release manifest");
    let compat_sig_exists = common::w18_rc_dir()
        .join("release_artifact_manifest.sig")
        .exists();
    let tool_journey_exists = common::w18_rc_dir()
        .join("golden_journey_tool_cli_report.json")
        .exists();
    let connector_journey_exists = common::w18_rc_dir()
        .join("golden_journey_connector_report.json")
        .exists();
    let positioning_guard =
        common::read_json(&common::w18_rc_dir().join("release_positioning_guard_report.json"));
    let positioning_pass =
        positioning_guard.get("status").and_then(JsonValue::as_str) == Some("PASS");

    let overall_pass = signature_summary.signature_verified
        && compat_sig_exists
        && tool_journey_exists
        && connector_journey_exists
        && positioning_pass
        && rl_all_pass;

    let readiness = json!({
        "schema": "ocl.w18.rc.release_readiness_report.v1",
        "run_manifest_ref": "target/ocl/w18/meta/run_manifest.json",
        "checks": [
            {"id":"release_artifact_manifest_exists","pass": manifest_path.exists()},
            {"id":"release_artifact_manifest_signature_exists","pass": canonical_sig_path.exists()},
            {"id":"release_artifact_manifest_signature_compat_exists","pass": compat_sig_exists},
            {"id":"release_artifact_manifest_signature_verified","pass": signature_summary.signature_verified},
            {"id":"golden_journey_tool_cli_present","pass": tool_journey_exists},
            {"id":"golden_journey_connector_present","pass": connector_journey_exists},
            {"id":"release_positioning_guard_pass","pass": positioning_pass},
            {"id":"rl_matrix_complete","pass": rl_required.len() == 17},
            {"id":"rl_matrix_all_pass","pass": rl_all_pass}
        ],
        "overall_pass": overall_pass
    });
    common::write_json_pretty(
        &common::w18_rc_dir().join("release_readiness_report.json"),
        &readiness,
    );

    assert!(
        overall_pass,
        "release_readiness_report overall_pass must be true"
    );
}
