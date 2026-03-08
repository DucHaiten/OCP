use serde_json::json;

#[path = "v19_gate_a_common.rs"]
mod v19;

#[test]
fn v19_settings_schema_contract_matches_package_defaults() {
    v19::ensure_run_manifest();

    let contract_path = v19::contracts_root()
        .join("editor")
        .join("ocp_settings_schema.v1.json");
    let contract = v19::read_json(&contract_path);
    let package = v19::editor_package_json();

    let contract_settings = contract
        .get("settings")
        .and_then(serde_json::Value::as_object)
        .expect("contract settings");
    let package_settings = package
        .get("contributes")
        .and_then(|v| v.get("configuration"))
        .and_then(|v| v.get("properties"))
        .and_then(serde_json::Value::as_object)
        .expect("package contributes.configuration.properties");

    assert_eq!(
        contract_settings.len(),
        package_settings.len(),
        "settings counts must match"
    );
    for (key, contract_meta) in contract_settings {
        let package_meta = package_settings
            .get(key)
            .unwrap_or_else(|| panic!("missing package setting `{key}`"));
        assert_eq!(
            contract_meta
                .get("type")
                .and_then(serde_json::Value::as_str)
                .expect("contract type"),
            package_meta
                .get("type")
                .and_then(serde_json::Value::as_str)
                .expect("package type"),
            "type mismatch for setting {key}"
        );
        assert_eq!(
            contract_meta.get("default").expect("contract default"),
            package_meta.get("default").expect("package default"),
            "default mismatch for setting {key}"
        );
    }

    let report = json!({
        "schema": "ocp.w19.editor.settings_schema_report.v1",
        "status": "PASS",
        "settings_count": contract_settings.len(),
        "contract_path": contract_path.to_string_lossy().replace('\\', "/"),
        "run_manifest_ref": "target/ocp/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report("editor/settings_schema_report.json", &report);
}
