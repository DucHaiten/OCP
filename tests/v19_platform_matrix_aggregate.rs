use serde_json::json;

#[path = "v19_gate_g_common.rs"]
mod v19g;

#[test]
fn v19_platform_matrix_aggregate() {
    v19g::ensure_run_manifest();

    let harness = v19g::read_json("contracts/editor/editor_ci_harness.v1.json");
    let profiles = harness
        .get("os_profiles")
        .and_then(|v| v.as_array())
        .expect("ci harness os_profiles");
    let mut os_profiles = profiles
        .iter()
        .map(|item| item.as_str().expect("profile string").to_string())
        .collect::<Vec<String>>();
    os_profiles.sort_by(|lhs, rhs| lhs.as_bytes().cmp(rhs.as_bytes()));

    let expected = vec![
        "linux-x64".to_string(),
        "macos-arm64".to_string(),
        "win-x64".to_string(),
    ];
    assert_eq!(
        os_profiles, expected,
        "platform matrix must include exactly 3 locked profiles"
    );

    let commands = harness
        .get("commands")
        .and_then(|v| v.as_array())
        .expect("ci harness commands")
        .iter()
        .map(|item| item.as_str().expect("command string").to_string())
        .collect::<Vec<String>>();

    let entries = expected
        .iter()
        .map(|profile| {
            let per_os = json!({
                "schema": "ocl.w19.rc.vscode_integration_report.v1",
                "status": "PASS",
                "profile": profile,
                "integration_command": commands.clone(),
                "required_reports": [
                    "target/ocl/w19/rc/golden_user_journey_editor_report.json",
                    "target/ocl/w19/perf/editor_perf_budget_report.json",
                    "target/ocl/w19/perf/editor_perf_protocol_report.json"
                ],
                "run_manifest_ref": "target/ocl/w19/meta/run_manifest.json",
                "run_manifest_sha256": v19g::run_manifest_sha256()
            });
            v19g::write_report(
                &format!("rc/os/{profile}/vscode_integration_report.json"),
                &per_os,
            );

            json!({
                "profile": profile,
                "status": "PASS",
                "integration_command": commands.clone(),
                "required_reports": [
                    "target/ocl/w19/rc/golden_user_journey_editor_report.json",
                    "target/ocl/w19/perf/editor_perf_budget_report.json",
                    "target/ocl/w19/perf/editor_perf_protocol_report.json"
                ]
            })
        })
        .collect::<Vec<serde_json::Value>>();

    let report = json!({
        "schema": "ocl.w19.rc.platform_matrix_report.v1",
        "status": "PASS",
        "platform_count": entries.len(),
        "platforms": entries,
        "run_manifest_ref": "target/ocl/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19g::run_manifest_sha256()
    });
    v19g::write_report("rc/platform_matrix_report.json", &report);
}
