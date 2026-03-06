#[path = "w17_gate_g_common.rs"]
mod w17;

use std::path::PathBuf;

use serde_json::json;

#[test]
fn v017_adoption_readiness_collects_cross_gate_evidence() {
    let required = w17::required_artifact_presence(&[
        (
            "contracts_sot",
            "target/ocl/w17/contracts/w17_contract_sot_report.json",
        ),
        (
            "trust_lifecycle",
            "target/ocl/w17/contracts/trust_lifecycle_report.json",
        ),
        (
            "compat_v016_line",
            "target/ocl/w17/compat/v016_line_compat_report.json",
        ),
        (
            "determinism_core",
            "target/ocl/w17/determinism/determinism_core_report.json",
        ),
        (
            "extension_governance",
            "target/ocl/w17/ecosystem/extension_governance_report.json",
        ),
        (
            "connector_baseline",
            "target/ocl/w17/connectors/connector_baseline_report.json",
        ),
    ]);

    let plan_path = PathBuf::from("OCP-OCL-MVP-PLAN-v0.17.md");
    let plan_text = w17::read_utf8(&plan_path);

    let gate_statuses = ["17-A", "17-B", "17-C", "17-D", "17-E", "17-F"]
        .iter()
        .map(|gate| {
            let status =
                w17::gate_status_in_plan(&plan_text, gate).unwrap_or_else(|| "-".to_string());
            json!({
                "gate": gate,
                "status": status
            })
        })
        .collect::<Vec<_>>();

    let gates_ready = gate_statuses.iter().all(|row| {
        row.get("status")
            .and_then(serde_json::Value::as_str)
            .map(|status| status == "DONE")
            .unwrap_or(false)
    });
    let artifacts_ready = w17::all_present(&required);

    let report = json!({
        "schema": "ocl.w17.adoption_readiness_report.v1",
        "run_manifest_ref": "target/ocl/w17/meta/run_manifest.json",
        "artifacts": required,
        "gates": gate_statuses,
        "checks": {
            "artifacts_ready": artifacts_ready,
            "gates_ready_through_17f": gates_ready
        },
        "pass": artifacts_ready && gates_ready
    });
    w17::write_rc_report("v017_adoption_readiness_report.json", &report);

    assert!(
        artifacts_ready,
        "required artifacts must exist before gate 17-G"
    );
    assert!(
        gates_ready,
        "gates 17-A..17-F must be DONE before gate 17-G"
    );
}
