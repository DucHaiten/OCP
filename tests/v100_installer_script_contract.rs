use std::fs;
use std::path::PathBuf;

use serde_json::{json, Value as JsonValue};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_json(path: PathBuf) -> JsonValue {
    serde_json::from_str(&fs::read_to_string(path).expect("read json")).expect("parse json")
}

fn write_report(rel_path: &str, value: &JsonValue) {
    let out = repo_root()
        .join("target")
        .join("ocp")
        .join("w100")
        .join(rel_path);
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).expect("create report parent");
    }
    fs::write(
        &out,
        serde_json::to_string_pretty(value).expect("serialize report"),
    )
    .expect("write report");
}

#[test]
fn v100_installer_script_contract() {
    let installer_contract = read_json(
        repo_root()
            .join("contracts")
            .join("release")
            .join("v1.0")
            .join("installer_contract_win.v1.json"),
    );
    let toolchain_contract = read_json(
        repo_root()
            .join("contracts")
            .join("release")
            .join("v1.0")
            .join("installer_toolchain_win.v1.json"),
    );

    let iss_path = repo_root()
        .join("installer")
        .join("windows")
        .join("ocp-win-x64.iss");
    let build_script_path = repo_root()
        .join("tools")
        .join("release")
        .join("build_win_installer.ps1");

    assert!(iss_path.exists(), "missing Inno Setup script");
    assert!(
        build_script_path.exists(),
        "missing Windows installer build script"
    );

    let iss = fs::read_to_string(&iss_path).expect("read iss");
    let build_script = fs::read_to_string(&build_script_path).expect("read build script");

    let installer_filename = installer_contract
        .get("installer_filename")
        .and_then(JsonValue::as_str)
        .unwrap_or("ocp-v1.0.0-setup-win-x64.exe");
    let icon_asset = installer_contract
        .get("installer_icon_asset")
        .and_then(JsonValue::as_str)
        .unwrap_or("installer/windows/ocp-installer.ico");
    let output_base = installer_filename.trim_end_matches(".exe");
    assert!(
        iss.contains(&format!("OutputBaseFilename={output_base}")),
        "installer script must emit expected filename"
    );

    let default_install_path = installer_contract
        .get("default_install_path")
        .and_then(JsonValue::as_str)
        .unwrap_or("C:\\Program Files\\OCP\\");
    let add_to_path_scope = installer_contract
        .get("add_to_path_scope")
        .and_then(JsonValue::as_str)
        .unwrap_or("system");
    let vscode_install_log_filename = installer_contract
        .get("vscode_install_log_filename")
        .and_then(JsonValue::as_str)
        .unwrap_or("vscode-extension-install.log");
    let vscode_manual_fallback_command = installer_contract
        .get("vscode_manual_fallback_command")
        .and_then(JsonValue::as_str)
        .unwrap_or("code --install-extension ocp-vscode-v1.0.0.vsix --force");
    let vscode_detection_modes = installer_contract
        .get("vscode_detection_modes")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect::<Vec<String>>();
    assert!(
        default_install_path.contains("OCP") && iss.contains("DefaultDirName={autopf}\\OCP"),
        "installer script must align with default install directory contract"
    );
    let icon_path = repo_root().join(icon_asset.replace('/', "\\"));
    assert!(
        icon_path.exists(),
        "missing installer icon asset {}",
        icon_path.display()
    );
    assert!(
        iss.contains("SetupIconFile="),
        "installer script must declare SetupIconFile"
    );

    if installer_contract
        .get("supports_install_dir_choice")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false)
    {
        assert!(
            iss.contains("DisableDirPage=no") && iss.contains("UsePreviousAppDir=no"),
            "installer script must explicitly show the install directory page and must not silently reuse a previous app directory"
        );
    }

    if installer_contract
        .get("supports_add_to_path")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false)
    {
        assert!(
            iss.contains("Name: \"addtopath\""),
            "installer script must expose add-to-PATH task"
        );
        match add_to_path_scope {
            "system" => assert!(
                iss.contains("Root: HKLM; Subkey: \"SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment\"")
                    && iss.contains("RegQueryStringValue(HKLM, 'SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment', 'Path', CurrentPath)"),
                "system PATH mode must write and query the machine PATH in HKLM"
            ),
            "user" => assert!(
                iss.contains("Root: HKCU; Subkey: \"Environment\"")
                    && iss.contains("RegQueryStringValue(HKCU, 'Environment', 'Path', CurrentPath)"),
                "user PATH mode must write and query the current-user PATH in HKCU"
            ),
            other => panic!("unsupported add_to_path_scope in contract: {other}"),
        }
    }
    if installer_contract
        .get("supports_add_to_path_confirmation")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false)
    {
        assert!(
            iss.contains("Add OCP CLI to PATH (recommended)"),
            "installer script must expose explicit add-to-PATH confirmation wording"
        );
    }

    if installer_contract
        .get("supports_uninstall")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false)
    {
        assert!(
            iss.contains("UninstallDisplayIcon"),
            "installer script must declare uninstall-aware metadata"
        );
    }

    assert!(
        iss.contains("Source: \"{#StageRoot}\\ocp.exe\""),
        "installer script must install staged ocp.exe"
    );
    assert!(
        iss.contains("Source: \"{#StageRoot}\\ocp-lsp.exe\""),
        "installer script must install staged ocp-lsp.exe"
    );
    assert!(
        iss.contains("Source: \"{#StageRoot}\\ocp-dap.exe\""),
        "installer script must install staged ocp-dap.exe"
    );
    assert!(
        iss.contains("Source: \"{#StageRoot}\\INSTALL-NEXT-STEPS.txt\""),
        "installer script must install next-steps note"
    );
    assert!(
        build_script.contains("render_installer_icon.py"),
        "build script must ensure installer icon asset exists"
    );
    assert!(
        build_script.contains("target\\release\\ocp-cli.exe"),
        "build script must resolve ocp-cli release binary"
    );
    assert!(
        build_script.contains("target\\release\\ocp-lsp.exe")
            && build_script.contains("target\\release\\ocp-dap.exe"),
        "build script must resolve ocp-lsp/ocp-dap release binaries"
    );
    assert!(
        build_script.contains("ISCC.exe"),
        "build script must invoke Inno Setup compiler"
    );
    assert!(
        build_script.contains("build_vsix.ps1"),
        "build script must resolve/build VSIX asset"
    );
    assert!(
        iss.contains("{#MyVsixName}") && iss.contains("installvscodeext"),
        "installer script must bundle VSIX and expose install task"
    );

    let vscode_install_mode = toolchain_contract
        .get("vscode_install_mode")
        .and_then(JsonValue::as_str)
        .unwrap_or("manual");
    if vscode_install_mode == "manual" {
        assert!(
            build_script.contains("VSCode extension is installed separately")
                && build_script.contains("ocp-vscode-v1.0.0.vsix"),
            "manual VSIX mode must be reflected in installer next steps"
        );
    } else if vscode_install_mode == "auto_if_code_cli_present" {
        assert!(
            iss.contains("GetVSCodeCliPath")
                && iss.contains("Microsoft VS Code\\bin\\code.cmd")
                && iss.contains("where code.cmd > ")
                && iss.contains("LoadStringFromFile")
                && !iss.contains("Result := 'code.cmd';")
                && build_script.contains(vscode_manual_fallback_command),
            "auto VSIX mode must detect VSCode install locations, resolve custom installs to an absolute code.cmd path, and provide manual fallback"
        );
        assert!(
            iss.contains(vscode_install_log_filename)
                && build_script.contains(vscode_install_log_filename),
            "auto VSIX mode must emit a stable VSCode install log filename in both installer script and next steps"
        );
        for mode in vscode_detection_modes {
            match mode.as_str() {
                "program_files" => assert!(
                    iss.contains("{pf}\\Microsoft VS Code\\bin\\code.cmd"),
                    "installer script must detect Program Files VSCode installs"
                ),
                "local_appdata" => assert!(
                    iss.contains("{localappdata}\\Programs\\Microsoft VS Code\\bin\\code.cmd"),
                    "installer script must detect LocalAppData VSCode installs"
                ),
                "path_lookup_absolute" => assert!(
                    iss.contains("where code.cmd > ")
                        && iss.contains("LoadStringFromFile")
                        && iss.contains("FileExists(DetectOutput)"),
                    "installer script must resolve an absolute code.cmd path from PATH lookup"
                ),
                other => panic!("unsupported vscode_detection_mode in contract: {other}"),
            }
        }
        assert!(
            iss.contains("Filename: \"{cmd}\"; Parameters: \"{code:GetVSCodeInstallCommand}\"")
                && iss.contains("function GetVSCodeInstallCommand(Param: string): string;")
                && iss.contains("'/C \"\"' + CliPath + '\" --install-extension \"'")
                && iss.contains("> \"' + LogPath + '\" 2>&1")
                && iss.contains("Flags: postinstall waituntilterminated runasoriginaluser"),
            "auto VSIX mode must invoke VSCode extension install through cmd /C under the original user context so the VSCode profile receives the extension"
        );
    }

    let report = json!({
        "schema": "ocp.w100.install.installer_script_contract_report.v1",
        "status": "PASS",
        "installer_script": "installer/windows/ocp-win-x64.iss",
        "build_script": "tools/release/build_win_installer.ps1",
        "installer_filename": installer_filename,
        "installer_icon_asset": icon_asset,
        "vscode_install_mode": vscode_install_mode,
        "run_manifest_ref": "target/ocp/w100/meta/run_manifest.json"
    });
    write_report("install/installer_script_contract_report.json", &report);
}
