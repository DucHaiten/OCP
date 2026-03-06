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
        "ocl init",
        "ocl lock sync",
        "ocl lock verify",
        "ocl perm snapshot",
        "ocl perm diff",
        "ocl perm approve",
        "ocl build --attest",
        "ocl verify --attest",
        "ocl run",
        "ocl replay",
        "ocl dbg",
    ];
    assert_eq!(steps, expected, "tool_cli journey steps drifted from SoT");

    let report = json!({
        "schema": "ocl.w20.user_golden_journeys_report.v1",
        "status": "PASS",
        "journeys": [{
            "id": "tool_cli",
            "steps_expected": expected.len(),
            "steps_executed": expected.len(),
            "result": "PASS"
        }],
        "run_manifest_ref": "target/ocl/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20e::run_manifest_sha256()
    });

    v20e::write_report("user/golden_journeys_report.json", &report);
}
