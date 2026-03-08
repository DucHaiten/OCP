use serde_json::json;

#[path = "v19_gate_a_common.rs"]
mod v19;

#[test]
fn v19_editor_versioning_policy_is_locked_and_valid() {
    v19::ensure_run_manifest();

    let path = v19::contracts_root()
        .join("editor")
        .join("editor_versioning_policy.v1.json");
    let value = v19::read_json(&path);

    assert_eq!(
        value
            .get("contract_id")
            .and_then(serde_json::Value::as_str)
            .expect("contract_id"),
        "editor.editor_versioning_policy"
    );
    assert_eq!(
        value
            .get("version")
            .and_then(serde_json::Value::as_str)
            .expect("version"),
        "v1"
    );

    let rules = value
        .get("bump_rules")
        .and_then(serde_json::Value::as_object)
        .expect("bump_rules");
    assert!(rules.contains_key("major"));
    assert!(rules.contains_key("minor"));
    assert!(rules.contains_key("patch"));

    let compat = value
        .get("compat_window_releases")
        .and_then(serde_json::Value::as_u64)
        .expect("compat_window_releases");
    assert!(compat >= 1, "compat_window_releases must be >= 1");

    let deprecation = value
        .get("deprecation_policy")
        .and_then(serde_json::Value::as_object)
        .expect("deprecation_policy");
    for field in [
        "settings_keys",
        "command_ids",
        "code_action_ids",
        "diagnostic_codes",
    ] {
        assert!(
            deprecation
                .get(field)
                .and_then(serde_json::Value::as_str)
                .is_some(),
            "missing deprecation policy field `{field}`"
        );
    }

    let report = json!({
        "schema": "ocp.w19.editor_versioning_policy_report.v1",
        "status": "PASS",
        "contract_path": path.to_string_lossy().replace('\\', "/"),
        "compat_window_releases": compat,
        "run_manifest_ref": "target/ocp/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report("contracts/editor_versioning_policy_report.json", &report);
}
