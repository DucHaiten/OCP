use serde_json::{json, Value as JsonValue};

#[path = "v100_gate_c_common.rs"]
mod v100c;

#[test]
fn v100_installer_toolchain_contract() {
    let contract = v100c::installer_toolchain_contract();
    assert_eq!(
        contract
            .get("contract_id")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "v1.installer_toolchain_win"
    );

    let builder = contract
        .get("builder")
        .and_then(JsonValue::as_str)
        .unwrap_or("");
    assert!(
        builder == "inno" || builder == "nsis" || builder == "wix",
        "builder must be one of inno|nsis|wix"
    );

    let signing_mode = contract
        .get("code_signing_mode")
        .and_then(JsonValue::as_str)
        .unwrap_or("");
    assert!(
        signing_mode == "none" || signing_mode == "authenticode",
        "code_signing_mode must be none|authenticode"
    );
    let vscode_install_mode = contract
        .get("vscode_install_mode")
        .and_then(JsonValue::as_str)
        .unwrap_or("");
    assert!(
        vscode_install_mode == "manual" || vscode_install_mode == "auto_if_code_cli_present",
        "vscode_install_mode must be manual|auto_if_code_cli_present"
    );

    if signing_mode == "none" {
        let vi = std::fs::read_to_string(
            v100c::repo_root()
                .join("docs")
                .join("vi")
                .join("security")
                .join("verify-download.md"),
        )
        .expect("read docs/vi/security/verify-download.md");
        let en = std::fs::read_to_string(
            v100c::repo_root()
                .join("docs")
                .join("en")
                .join("security")
                .join("verify-download.md"),
        )
        .expect("read docs/en/security/verify-download.md");
        assert!(
            vi.contains("SmartScreen") || vi.contains("code_signing_mode=none"),
            "vi verify guide must warn when code_signing_mode=none"
        );
        assert!(
            en.contains("SmartScreen") || en.contains("code_signing_mode=none"),
            "en verify guide must warn when code_signing_mode=none"
        );
    }

    let report = json!({
        "schema": "ocp.w100.install.installer_toolchain_report.v1",
        "status": "PASS",
        "builder": builder,
        "code_signing_mode": signing_mode,
        "vscode_install_mode": vscode_install_mode,
        "run_manifest_ref": "target/ocp/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100c::run_manifest_sha256()
    });
    v100c::write_report("install/installer_toolchain_report.json", &report);
}
