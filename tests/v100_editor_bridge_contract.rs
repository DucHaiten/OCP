use std::fs;
use std::path::PathBuf;

use serde_json::json;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn write_report(rel_path: &str, value: &serde_json::Value) {
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
fn v100_editor_bridge_contract() {
    let commands_ts = fs::read_to_string(
        repo_root()
            .join("editor")
            .join("vscode")
            .join("ocp")
            .join("src")
            .join("commands.ts"),
    )
    .expect("read commands.ts");
    let commands_js = fs::read_to_string(
        repo_root()
            .join("editor")
            .join("vscode")
            .join("ocp")
            .join("dist")
            .join("commands.js"),
    )
    .expect("read commands.js");
    let lsp_client = fs::read_to_string(
        repo_root()
            .join("editor")
            .join("vscode")
            .join("ocp")
            .join("src")
            .join("lspClient.ts"),
    )
    .expect("read lspClient.ts");
    let runtime_paths = fs::read_to_string(
        repo_root()
            .join("editor")
            .join("vscode")
            .join("ocp")
            .join("src")
            .join("runtimePaths.ts"),
    )
    .expect("read runtimePaths.ts");
    let dap_runtime = fs::read_to_string(
        repo_root()
            .join("editor")
            .join("vscode")
            .join("ocp")
            .join("src")
            .join("dapRuntime.ts"),
    )
    .expect("read dapRuntime.ts");
    let package_json = fs::read_to_string(
        repo_root()
            .join("editor")
            .join("vscode")
            .join("ocp")
            .join("package.json"),
    )
    .expect("read package.json");
    let build_vsix = fs::read_to_string(
        repo_root()
            .join("tools")
            .join("release")
            .join("build_vsix.ps1"),
    )
    .expect("read build_vsix.ps1");

    for source in [&commands_ts, &commands_js] {
        assert!(
            source.contains("execFile"),
            "editor command bridge must execute CLI commands"
        );
        assert!(
            source.contains("ocp.formatDocument")
                && source.contains("ocp.checkWorkspace")
                && source.contains("ocp.openDoctorReport")
                && source.contains("ocp.openFixPlan")
                && source.contains("ocp.openDebugTrace"),
            "editor command bridge must register all public command ids"
        );
        assert!(
            !source.contains("will be fully wired in later gates"),
            "placeholder command bridge message must be removed"
        );
    }
    assert!(
        runtime_paths.contains("bin")
            && runtime_paths.contains("win-x64")
            && commands_ts.contains("\"ocp.exe\"")
            && lsp_client.contains("\"ocp-lsp.exe\"")
            && dap_runtime.contains("\"ocp-dap.exe\""),
        "runtime path resolver and callers must know bundled CLI/LSP/DAP names"
    );

    assert!(
        lsp_client.contains("workspace is untrusted")
            && lsp_client.contains("ocp-lsp.exe")
            && lsp_client.contains("registerHoverProvider")
            && lsp_client.contains("registerDocumentSemanticTokensProvider")
            && lsp_client.contains("spawn("),
        "lsp client bootstrap must reflect trust gating and real bundled LSP runtime wiring"
    );
    assert!(
        dap_runtime.contains("registerDebugAdapterDescriptorFactory")
            && dap_runtime.contains("registerDebugConfigurationProvider")
            && dap_runtime.contains("ocp-dap.exe"),
        "debug runtime must register a real bundled DAP adapter"
    );
    assert!(
        package_json.contains("\"debuggers\"") && package_json.contains("\"type\": \"ocp\""),
        "package.json must contribute OCP debugger metadata"
    );

    assert!(
        build_vsix.contains("Resolve-BinarySource")
            && build_vsix.contains("bin\\win-x64")
            && build_vsix.contains("ocp.exe")
            && build_vsix.contains("ocp-lsp.exe")
            && build_vsix.contains("ocp-dap.exe"),
        "VSIX build script must bundle CLI/LSP/DAP into extension/bin/win-x64"
    );

    let report = json!({
        "schema": "ocp.w100.editor.bridge_contract_report.v1",
        "status": "PASS",
        "commands_source": "editor/vscode/ocp/src/commands.ts",
        "lsp_client_source": "editor/vscode/ocp/src/lspClient.ts",
        "dap_runtime_source": "editor/vscode/ocp/src/dapRuntime.ts",
        "vsix_build_script": "tools/release/build_vsix.ps1",
        "bundled_binaries_rel": [
            "extension/bin/win-x64/ocp.exe",
            "extension/bin/win-x64/ocp-lsp.exe",
            "extension/bin/win-x64/ocp-dap.exe"
        ]
    });
    write_report("editor/editor_bridge_contract_report.json", &report);
}
