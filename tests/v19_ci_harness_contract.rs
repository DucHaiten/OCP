use serde_json::json;

#[path = "v19_gate_g_common.rs"]
mod v19g;

#[test]
fn v19_ci_harness_contract() {
    v19g::ensure_run_manifest();

    let harness = v19g::read_json("contracts/editor/editor_ci_harness.v1.json");
    let schema = harness
        .get("schema")
        .and_then(|v| v.as_str())
        .expect("schema");
    assert_eq!(schema, "ocl.editor.ci_harness.v1");

    let profiles = harness
        .get("os_profiles")
        .and_then(|v| v.as_array())
        .expect("os_profiles");
    let mut names = profiles
        .iter()
        .map(|item| item.as_str().expect("profile string").to_string())
        .collect::<Vec<String>>();
    names.sort_by(|lhs, rhs| lhs.as_bytes().cmp(rhs.as_bytes()));
    assert_eq!(
        names,
        vec![
            "linux-x64".to_string(),
            "macos-arm64".to_string(),
            "win-x64".to_string(),
        ]
    );

    let commands = harness
        .get("commands")
        .and_then(|v| v.as_array())
        .expect("commands");
    let has_integration = commands
        .iter()
        .any(|item| item.as_str() == Some("pnpm --dir editor/vscode/ocp-ocl run test:integration"));
    assert!(has_integration, "missing integration command contract");

    let required_fields = harness
        .get("provenance_required_fields")
        .and_then(|v| v.as_array())
        .expect("provenance_required_fields");
    for field in ["run_id", "job_id", "git_commit"] {
        let found = required_fields
            .iter()
            .any(|item| item.as_str() == Some(field));
        assert!(found, "missing provenance field {field}");
    }

    let report = json!({
        "schema": "ocl.w19.rc.ci_harness_report.v1",
        "status": "PASS",
        "os_profiles": names,
        "commands": commands,
        "provenance_required_fields": required_fields,
        "run_manifest_ref": "target/ocl/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19g::run_manifest_sha256()
    });
    v19g::write_report("rc/ci_harness_report.json", &report);
}
