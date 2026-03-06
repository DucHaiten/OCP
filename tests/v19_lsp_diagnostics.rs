use std::fs;

use ocl_sdk::lsp_diagnostics_from_source_v19;
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
fn v19_lsp_diagnostics_realtime_core() {
    v19::ensure_run_manifest();

    let bad = fixture("diagnostics_error.ocl");
    let ok = fixture("diagnostics_ok.ocl");

    let bad_diags = lsp_diagnostics_from_source_v19(&bad, 19);
    let ok_diags = lsp_diagnostics_from_source_v19(&ok, 19);

    assert!(
        !bad_diags.is_empty(),
        "invalid source must produce at least one diagnostic"
    );
    assert!(
        ok_diags.is_empty(),
        "valid source must not produce diagnostic"
    );

    let report = json!({
        "schema": "ocl.w19.lsp.diagnostics_report.v1",
        "status": "PASS",
        "bad_diagnostics": bad_diags,
        "ok_diagnostics_count": ok_diags.len(),
        "run_manifest_ref": "target/ocl/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report("lsp/diagnostics_report.json", &report);
}
