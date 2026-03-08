use std::fs;

use ocp_sdk::lsp_symbols_from_source_v19;
use serde_json::json;

#[path = "v19_gate_a_common.rs"]
mod v19;

#[test]
fn v19_lsp_symbols_core() {
    v19::ensure_run_manifest();
    let path = v19::repo_root()
        .join("tests")
        .join("fixtures")
        .join("v19")
        .join("lsp")
        .join("symbols.ocp");
    let source = fs::read_to_string(&path).unwrap_or_else(|_| panic!("read {}", path.display()));

    let symbols = lsp_symbols_from_source_v19(&source, 19).expect("parse symbols fixture");
    let names = symbols
        .iter()
        .map(|s| s.name.clone())
        .collect::<Vec<String>>();

    assert!(names.iter().any(|n| n == "User"), "missing struct symbol");
    assert!(names.iter().any(|n| n == "Status"), "missing enum symbol");
    assert!(names.iter().any(|n| n == "render"), "missing fn symbol");

    let report = json!({
        "schema": "ocp.w19.lsp.symbols_report.v1",
        "status": "PASS",
        "symbol_count": symbols.len(),
        "symbols": symbols,
        "run_manifest_ref": "target/ocp/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report("lsp/symbols_report.json", &report);
}
