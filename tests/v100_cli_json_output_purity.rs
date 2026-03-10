use serde_json::Value as JsonValue;

#[path = "v100_cli_extension_common.rs"]
mod common;

#[path = "w17_gate_b_cli_common.rs"]
mod cli_common;

#[test]
fn v100_cli_json_output_purity_with_legacy_warning() {
    let root = common::setup_legacy_oc_project("v100_json_output_purity");
    let root_s = root.to_string_lossy().to_string();
    let check = cli_common::run_ocp_cli(&["check", &root_s, "--json"]);
    let stdout = cli_common::assert_success(&check);
    let stderr = String::from_utf8_lossy(&check.stderr);
    assert!(
        !stderr.contains("W-LEGACY-OCP-EXTENSION"),
        "--json mode must not emit plain-text warning to stderr, got: {stderr}"
    );

    let parsed: JsonValue = serde_json::from_str(&stdout).expect("strict json payload");
    assert_eq!(
        parsed.get("ok").and_then(JsonValue::as_bool),
        Some(true),
        "check --json must report ok=true"
    );
    let warnings = parsed
        .get("warnings")
        .and_then(JsonValue::as_array)
        .expect("warnings array");
    assert!(
        warnings.iter().any(|item| {
            item.get("code").and_then(JsonValue::as_str) == Some("W-LEGACY-OCP-EXTENSION")
        }),
        "warnings payload must include legacy extension warning code"
    );
}
