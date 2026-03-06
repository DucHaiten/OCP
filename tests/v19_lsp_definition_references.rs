use std::fs;

use ocl_sdk::{lsp_definition_locations_v19, lsp_reference_locations_v19};
use serde_json::json;

#[path = "v19_gate_a_common.rs"]
mod v19;

#[test]
fn v19_lsp_definition_references_core() {
    v19::ensure_run_manifest();

    let fixture_path = v19::repo_root()
        .join("tests")
        .join("fixtures")
        .join("v19")
        .join("lsp")
        .join("navigation.ocl");
    let source = fs::read_to_string(&fixture_path)
        .unwrap_or_else(|_| panic!("read {}", fixture_path.display()));

    let defs = lsp_definition_locations_v19(&source, 19, "helper").expect("definition parse");
    let refs = lsp_reference_locations_v19(&source, 19, "helper").expect("references parse");

    assert_eq!(defs.len(), 1, "helper must have exactly one definition");
    assert!(
        !refs.is_empty(),
        "helper must have at least one reference in navigation fixture"
    );
    assert_eq!(defs[0].line, 1, "helper definition line must be stable");

    let report = json!({
        "schema": "ocl.w19.lsp.navigation_report.v1",
        "status": "PASS",
        "phase": "definition_references",
        "definitions": defs,
        "references": refs,
        "run_manifest_ref": "target/ocl/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report("lsp/navigation_report.json", &report);
}
