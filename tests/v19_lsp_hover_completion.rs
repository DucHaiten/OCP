use std::fs;

use ocl_sdk::{lsp_completion_items_from_source_v19, lsp_hover_for_symbol_v19};
use serde_json::json;

#[path = "v19_gate_a_common.rs"]
mod v19;

fn fixture(rel: &str) -> String {
    let path = v19::repo_root()
        .join("tests")
        .join("fixtures")
        .join("v19")
        .join("lsp")
        .join(rel);
    fs::read_to_string(&path).unwrap_or_else(|_| panic!("read {}", path.display()))
}

#[test]
fn v19_lsp_hover_completion_core() {
    v19::ensure_run_manifest();

    let caps_path = v19::contracts_root()
        .join("editor")
        .join("ocl_lsp_capabilities.v1.json");
    let caps = v19::read_json(&caps_path);
    let capabilities = caps
        .get("capabilities")
        .and_then(serde_json::Value::as_array)
        .expect("capabilities array");
    assert!(
        capabilities
            .iter()
            .any(|item| item.as_str() == Some("textDocument/hover")),
        "hover capability must be declared"
    );
    assert!(
        capabilities
            .iter()
            .any(|item| item.as_str() == Some("textDocument/completion")),
        "completion capability must be declared"
    );

    let source = fixture("navigation.ocl");
    let hover = lsp_hover_for_symbol_v19(&source, 19, "helper")
        .expect("hover parse")
        .expect("hover helper");
    assert_eq!(hover.label, "helper");
    assert_eq!(hover.kind, "function");

    let completions = lsp_completion_items_from_source_v19(&source, 19).expect("completion parse");
    assert!(
        completions.iter().any(|item| item.label == "helper"),
        "completion must include helper"
    );
    assert!(
        completions.iter().any(|item| item.label == "main"),
        "completion must include main"
    );

    let report = json!({
        "schema": "ocl.w19.lsp.navigation_report.v1",
        "status": "PASS",
        "phase": "hover_completion",
        "hover": hover,
        "completion_count": completions.len(),
        "run_manifest_ref": "target/ocl/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report("lsp/navigation_report.json", &report);
}
