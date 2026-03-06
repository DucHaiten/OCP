use std::collections::BTreeSet;
use std::fs;

use ocl_sdk::lsp_diagnostics_from_source_v19;
use serde_json::json;

#[path = "v19_gate_a_common.rs"]
mod v19;

#[test]
fn v19_diagnostics_taxonomy_mapping_contract_is_enforced() {
    v19::ensure_run_manifest();

    let contract_path = v19::contracts_root()
        .join("editor")
        .join("ocl_diagnostics_mapping.v1.json");
    let contract = v19::read_json(&contract_path);
    let severities = contract
        .get("severity_map")
        .and_then(serde_json::Value::as_object)
        .expect("severity_map object")
        .values()
        .filter_map(serde_json::Value::as_str)
        .map(ToOwned::to_owned)
        .collect::<BTreeSet<String>>();

    let source_path = v19::repo_root()
        .join("tests")
        .join("fixtures")
        .join("v19")
        .join("lsp")
        .join("diagnostics_error.ocl");
    let source = fs::read_to_string(&source_path)
        .unwrap_or_else(|_| panic!("read {}", source_path.display()));
    let diagnostics = lsp_diagnostics_from_source_v19(&source, 19);
    assert!(!diagnostics.is_empty());
    for diag in &diagnostics {
        assert!(
            severities.contains(&diag.severity),
            "severity `{}` not declared in taxonomy mapping contract",
            diag.severity
        );
    }

    let report = json!({
        "schema": "ocl.w19.lsp.diagnostics_taxonomy_mapping_report.v1",
        "status": "PASS",
        "diagnostic_count": diagnostics.len(),
        "mapping_contract_path": contract_path.to_string_lossy().replace('\\', "/"),
        "run_manifest_ref": "target/ocl/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report("lsp/diagnostics_taxonomy_mapping_report.json", &report);
}
