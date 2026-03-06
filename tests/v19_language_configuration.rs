use serde_json::json;

#[path = "v19_gate_a_common.rs"]
mod v19;

#[test]
fn v19_language_configuration_matches_language_profile_contract() {
    v19::ensure_run_manifest();

    let profile_path = v19::contracts_root()
        .join("editor")
        .join("ocl_language_profile.v1.json");
    let profile = v19::read_json(&profile_path);
    let config_path = v19::repo_root()
        .join("editor")
        .join("vscode")
        .join("ocp-ocl")
        .join("language-configuration.json");
    let config = v19::read_json(&config_path);

    let line_comment = config
        .get("comments")
        .and_then(|v| v.get("lineComment"))
        .and_then(serde_json::Value::as_str)
        .expect("line comment");
    assert_eq!(
        line_comment,
        profile
            .get("comment_tokens")
            .and_then(|v| v.get("line"))
            .and_then(serde_json::Value::as_str)
            .expect("profile comment_tokens.line")
    );

    let config_brackets = v19::sorted_strings_from_json_array(&serde_json::Value::Array(
        config
            .get("brackets")
            .and_then(serde_json::Value::as_array)
            .expect("config brackets")
            .iter()
            .map(|pair| {
                let arr = pair.as_array().expect("bracket pair");
                serde_json::Value::String(format!(
                    "{}{}",
                    arr[0].as_str().expect("open"),
                    arr[1].as_str().expect("close")
                ))
            })
            .collect::<Vec<serde_json::Value>>(),
    ));
    let profile_brackets = v19::sorted_strings_from_json_array(&serde_json::Value::Array(
        profile
            .get("bracket_pairs")
            .and_then(serde_json::Value::as_array)
            .expect("profile bracket_pairs")
            .iter()
            .map(|pair| {
                let arr = pair.as_array().expect("bracket pair");
                serde_json::Value::String(format!(
                    "{}{}",
                    arr[0].as_str().expect("open"),
                    arr[1].as_str().expect("close")
                ))
            })
            .collect::<Vec<serde_json::Value>>(),
    ));
    assert_eq!(
        config_brackets, profile_brackets,
        "bracket pairs must match profile contract"
    );

    let report = json!({
        "schema": "ocl.w19.editor.language_configuration_report.v1",
        "status": "PASS",
        "config_path": config_path.to_string_lossy().replace('\\', "/"),
        "profile_path": profile_path.to_string_lossy().replace('\\', "/"),
        "run_manifest_ref": "target/ocl/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report("editor/language_configuration_report.json", &report);
}
