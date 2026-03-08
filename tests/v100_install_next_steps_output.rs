use serde_json::Value as JsonValue;

#[path = "v100_gate_c_common.rs"]
mod v100c;

#[test]
fn v100_install_next_steps_output() {
    let report = v100c::ensure_win_installer_smoke_report();
    let steps = report
        .get("next_steps")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect::<Vec<String>>();
    assert!(
        steps.iter().any(|s| s.contains("ocp --version")),
        "next_steps must include `ocp --version`"
    );
    assert!(
        steps.iter().any(|s| s.contains("ocp init hello")),
        "next_steps must include `ocp init hello`"
    );
    assert!(
        steps
            .iter()
            .any(|s| s.contains("verify-download.md") || s.contains("Verify Download")),
        "next_steps must point to verify-download guidance"
    );
    assert!(
        steps
            .iter()
            .any(|s| s.contains("vscode-extension-install.log")),
        "next_steps must mention the VSCode auto-install log file"
    );
    assert!(
        steps
            .iter()
            .any(|s| s.contains("Manual fallback: code --install-extension")),
        "next_steps must mention manual VSIX fallback"
    );
}
