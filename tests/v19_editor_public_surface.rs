use serde_json::json;

#[path = "v19_gate_a_common.rs"]
mod v19;

fn sorted_command_ids(package: &serde_json::Value) -> Vec<String> {
    let mut out = package
        .get("contributes")
        .and_then(|v| v.get("commands"))
        .and_then(serde_json::Value::as_array)
        .expect("package commands")
        .iter()
        .map(|item| {
            item.get("command")
                .and_then(serde_json::Value::as_str)
                .expect("command id")
                .to_string()
        })
        .collect::<Vec<String>>();
    out.sort_by(|lhs, rhs| lhs.as_bytes().cmp(rhs.as_bytes()));
    out
}

fn sorted_setting_keys(package: &serde_json::Value) -> Vec<String> {
    let props = package
        .get("contributes")
        .and_then(|v| v.get("configuration"))
        .and_then(|v| v.get("properties"))
        .and_then(serde_json::Value::as_object)
        .expect("configuration properties");
    let mut out = props.keys().cloned().collect::<Vec<String>>();
    out.sort_by(|lhs, rhs| lhs.as_bytes().cmp(rhs.as_bytes()));
    out
}

#[test]
fn v19_editor_public_surface_matches_extension_contributions() {
    v19::ensure_run_manifest();

    let contract_path = v19::contracts_root()
        .join("editor")
        .join("editor_public_surface.v1.json");
    let contract = v19::read_json(&contract_path);
    let package = v19::editor_package_json();

    let contract_language = contract
        .get("language_id")
        .and_then(serde_json::Value::as_str)
        .expect("contract language_id");
    let package_language = package
        .get("contributes")
        .and_then(|v| v.get("languages"))
        .and_then(serde_json::Value::as_array)
        .expect("languages")
        .first()
        .expect("first language")
        .get("id")
        .and_then(serde_json::Value::as_str)
        .expect("language id");
    assert_eq!(contract_language, package_language);

    let contract_ext = v19::sorted_strings_from_json_array(
        contract.get("extensions").expect("contract extensions"),
    );
    let package_ext = v19::sorted_strings_from_json_array(
        package
            .get("contributes")
            .and_then(|v| v.get("languages"))
            .and_then(serde_json::Value::as_array)
            .expect("languages")
            .first()
            .expect("first language")
            .get("extensions")
            .expect("language extensions"),
    );
    assert_eq!(contract_ext, package_ext);

    let contract_commands = v19::sorted_strings_from_json_array(
        contract.get("command_ids").expect("contract command_ids"),
    );
    let package_commands = sorted_command_ids(&package);
    assert_eq!(contract_commands, package_commands);

    let contract_settings = v19::sorted_strings_from_json_array(
        contract
            .get("settings_keys")
            .expect("contract settings_keys"),
    );
    let package_settings = sorted_setting_keys(&package);
    assert_eq!(contract_settings, package_settings);

    let report = json!({
        "schema": "ocl.w19.editor_public_surface_report.v1",
        "status": "PASS",
        "contract_path": contract_path.to_string_lossy().replace('\\', "/"),
        "language_id": contract_language,
        "command_ids": contract_commands,
        "settings_keys": contract_settings,
        "run_manifest_ref": "target/ocl/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report("contracts/editor_public_surface_report.json", &report);
}
