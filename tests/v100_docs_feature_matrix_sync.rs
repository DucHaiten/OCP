use serde_json::{json, Value as JsonValue};

#[path = "v100_gate_d_common.rs"]
mod v100d;

#[test]
fn v100_docs_feature_matrix_sync() {
    v100d::ensure_run_manifest();

    let matrix_contract = v100d::docs_contract_feature_matrix();
    assert_eq!(
        matrix_contract
            .get("contract_id")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "v1.docs_feature_matrix"
    );
    let matrix_file = matrix_contract
        .get("required_matrix_file")
        .and_then(JsonValue::as_str)
        .unwrap_or("docs/vi/USER_GUIDE.md");
    let matrix_path = v100d::repo_root().join(matrix_file);
    assert!(matrix_path.exists(), "missing {}", matrix_path.display());
    let matrix = v100d::read_text(&matrix_path);

    let cli_surface = v100d::docs_contract_cli_surface();
    let mut commands_verified = Vec::<String>::new();
    for cmd in cli_surface
        .get("canonical_commands")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default()
        .iter()
        .filter_map(JsonValue::as_str)
    {
        assert!(
            matrix.contains(cmd),
            "user guide missing canonical command `{cmd}`"
        );
        commands_verified.push(cmd.to_string());
    }

    let report = json!({
        "schema": "ocp.w100.docs.feature_matrix_sync_report.v1",
        "status": "PASS",
        "matrix_file": matrix_file,
        "commands_verified": commands_verified,
        "sync_policy": matrix_contract.get("sync_policy").cloned().unwrap_or(JsonValue::Null),
        "run_manifest_ref": "target/ocp/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100d::run_manifest_sha256()
    });
    v100d::write_report("docs/feature_matrix_sync_report.json", &report);
}
