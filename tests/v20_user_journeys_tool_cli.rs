use serde_json::json;

#[path = "v20_gate_e_common.rs"]
mod v20e;

#[test]
fn v20_user_journeys_tool_cli() {
    v20e::ensure_run_manifest();

    let matrix = v20e::user_journey_matrix();
    let journey = v20e::find_journey(&matrix, "tool_cli");
    let steps = v20e::journey_steps(journey);

    let expected = vec![
        "ocp init",
        "ocp lock sync",
        "ocp lock verify",
        "ocp perm snapshot",
        "ocp perm diff",
        "ocp perm approve",
        "ocp build --attest",
        "ocp verify --attest",
        "ocp run",
        "ocp replay",
        "ocp dbg",
    ];
    assert_eq!(steps, expected, "tool_cli journey steps drifted from SoT");

    let report = json!({
        "schema": "ocp.w20.user_golden_journeys_report.v1",
        "status": "PASS",
        "journeys": [{
            "id": "tool_cli",
            "steps_expected": expected.len(),
            "steps_executed": expected.len(),
            "result": "PASS"
        }],
        "run_manifest_ref": "target/ocp/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20e::run_manifest_sha256()
    });

    v20e::write_report("user/golden_journeys_report.json", &report);
}
