use std::fs;

use serde_json::Value as JsonValue;

#[path = "v100_cli_extension_common.rs"]
mod common;

#[path = "w17_gate_b_cli_common.rs"]
mod cli_common;

#[test]
fn v100_cli_json_error_payload_includes_callsite_context() {
    let root = common::init_project_with_template("v100_json_error_callsite", "tool-cli");
    fs::write(root.join("src").join("main.ocp"), "let x = ;\n").expect("write broken source");
    let root_s = root.to_string_lossy().to_string();

    let out = cli_common::run_ocp_cli(&["check", &root_s, "--json"]);
    assert!(
        !out.status.success(),
        "check --json should fail for broken source:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let parsed: JsonValue = serde_json::from_str(&stdout).expect("parse strict error json");
    let error = parsed.get("error").expect("error object");
    let callsite = error.get("callsite").expect("error.callsite");
    assert_eq!(
        callsite.get("module").and_then(JsonValue::as_str),
        Some("file_id:1")
    );
    assert!(
        callsite
            .get("line")
            .and_then(JsonValue::as_u64)
            .unwrap_or_default()
            > 0,
        "callsite.line must be positive"
    );
    assert!(
        callsite
            .get("column")
            .and_then(JsonValue::as_u64)
            .unwrap_or_default()
            > 0,
        "callsite.column must be positive"
    );
}
