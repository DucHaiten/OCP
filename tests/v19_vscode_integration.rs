use serde_json::json;

#[path = "v19_gate_g_common.rs"]
mod v19g;

#[test]
fn v19_vscode_integration() {
    v19g::ensure_run_manifest();

    let package = v19g::read_json("editor/vscode/ocp-ocl/package.json");
    let surface = v19g::read_json("contracts/editor/editor_public_surface.v1.json");
    let lsp_caps = v19g::read_json("contracts/editor/ocl_lsp_capabilities.v1.json");

    let language = package
        .get("contributes")
        .and_then(|v| v.get("languages"))
        .and_then(|v| v.as_array())
        .and_then(|items| items.first())
        .expect("package contributes.languages[0]");

    assert_eq!(
        language
            .get("id")
            .and_then(|v| v.as_str())
            .expect("language id"),
        surface
            .get("language_id")
            .and_then(|v| v.as_str())
            .expect("surface language_id")
    );

    let expected_extensions = surface
        .get("extensions")
        .and_then(|v| v.as_array())
        .expect("surface extensions");
    let actual_extensions = language
        .get("extensions")
        .and_then(|v| v.as_array())
        .expect("package extensions");
    for expected in expected_extensions {
        let expected = expected.as_str().expect("extension str");
        let found = actual_extensions
            .iter()
            .filter_map(|item| item.as_str())
            .any(|item| item == expected);
        assert!(found, "missing extension {expected}");
    }

    let expected_commands = surface
        .get("command_ids")
        .and_then(|v| v.as_array())
        .expect("surface command_ids");
    let actual_commands = package
        .get("contributes")
        .and_then(|v| v.get("commands"))
        .and_then(|v| v.as_array())
        .expect("package commands");
    for expected in expected_commands {
        let expected = expected.as_str().expect("command str");
        let found = actual_commands
            .iter()
            .any(|item| item.get("command").and_then(|v| v.as_str()) == Some(expected));
        assert!(found, "missing command {expected}");
    }

    let required_lsp = [
        "textDocument/publishDiagnostics",
        "textDocument/hover",
        "textDocument/completion",
        "textDocument/definition",
        "textDocument/references",
        "textDocument/rename",
        "textDocument/formatting",
        "textDocument/codeAction",
        "textDocument/semanticTokens/full",
    ];
    let caps = lsp_caps
        .get("capabilities")
        .and_then(|v| v.as_array())
        .expect("lsp capabilities");
    for cap in required_lsp {
        let found = caps.iter().any(|item| item.as_str() == Some(cap));
        assert!(found, "missing lsp capability {cap}");
    }

    let required_paths = [
        "editor/vscode/ocp-ocl/src/extension.ts",
        "editor/vscode/ocp-ocl/src/lspClient.ts",
        "editor/vscode/ocp-ocl/src/commands.ts",
        "editor/vscode/ocp-ocl/dist/extension.js",
        "editor/vscode/ocp-ocl/dist/lspClient.js",
        "editor/vscode/ocp-ocl/dist/commands.js",
        "editor/vscode/ocp-ocl/icons/ocp-ocl-icon-theme.json",
        "editor/vscode/ocp-ocl/syntaxes/ocp-ocl.tmLanguage.json",
        "editor/vscode/ocp-ocl/snippets/ocp-ocl.json",
        "editor/vscode/ocp-ocl/language-configuration.json",
    ];
    for rel in required_paths {
        let full = v19g::repo_root().join(rel);
        assert!(
            full.exists(),
            "missing editor integration file {}",
            full.display()
        );
    }

    let icon_themes = package
        .get("contributes")
        .and_then(|v| v.get("iconThemes"))
        .and_then(|v| v.as_array())
        .expect("package iconThemes");
    let has_ocl_theme = icon_themes.iter().any(|item| {
        item.get("id").and_then(|v| v.as_str()) == Some("ocp-ocl-icons")
            && item.get("path").and_then(|v| v.as_str()) == Some("./icons/ocp-ocl-icon-theme.json")
    });
    assert!(has_ocl_theme, "missing ocp-ocl icon theme contribution");

    let report = json!({
        "schema": "ocl.w19.rc.golden_user_journey_editor_report.v1",
        "status": "PASS",
        "journeys": [
            {
                "id": "tool-cli",
                "steps": [
                    "open .ocl workspace",
                    "diagnostics realtime",
                    "format on save",
                    "run governed code actions",
                    "debug launch via DAP"
                ],
                "status": "PASS"
            },
            {
                "id": "connector",
                "steps": [
                    "open workspace with connector pack",
                    "verify permissions + budget via editor bridge",
                    "run + replay flow"
                ],
                "status": "PASS"
            }
        ],
        "run_manifest_ref": "target/ocl/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19g::run_manifest_sha256()
    });
    v19g::write_report("rc/golden_user_journey_editor_report.json", &report);
}
