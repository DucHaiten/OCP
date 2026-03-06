use serde_json::json;

#[path = "v19_gate_a_common.rs"]
mod v19;

#[test]
fn v19_editor_language_profile_matches_extension_manifest() {
    v19::ensure_run_manifest();

    let contract_path = v19::contracts_root()
        .join("editor")
        .join("ocl_language_profile.v1.json");
    let contract = v19::read_json(&contract_path);
    let package = v19::editor_package_json();

    let language_id = contract
        .get("language_id")
        .and_then(serde_json::Value::as_str)
        .expect("contract language_id");
    assert_eq!(language_id, "ocp-ocl");

    let contract_ext = v19::sorted_strings_from_json_array(
        contract.get("extensions").expect("contract extensions"),
    );

    let lang_entry = package
        .get("contributes")
        .and_then(|v| v.get("languages"))
        .and_then(serde_json::Value::as_array)
        .expect("package contributes.languages")
        .iter()
        .find(|entry| {
            entry
                .get("id")
                .and_then(serde_json::Value::as_str)
                .map(|id| id == language_id)
                .unwrap_or(false)
        })
        .expect("language entry for ocp-ocl");

    let package_ext = v19::sorted_strings_from_json_array(
        lang_entry
            .get("extensions")
            .expect("package language extensions"),
    );
    assert_eq!(
        contract_ext, package_ext,
        "extensions must match language profile contract"
    );

    let report = json!({
        "schema": "ocl.w19.editor.language_profile_report.v1",
        "status": "PASS",
        "language_id": language_id,
        "extensions": contract_ext,
        "contract_path": contract_path.to_string_lossy().replace('\\', "/"),
        "run_manifest_ref": "target/ocl/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report("editor/language_profile_report.json", &report);
}
