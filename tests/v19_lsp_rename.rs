use std::fs;

use ocp_sdk::lsp_rename_preview_v19;
use serde_json::json;

#[path = "v19_gate_a_common.rs"]
mod v19;

#[test]
fn v19_lsp_rename_preview_is_deterministic() {
    v19::ensure_run_manifest();

    let fixture_path = v19::repo_root()
        .join("tests")
        .join("fixtures")
        .join("v19")
        .join("lsp")
        .join("navigation.ocp");
    let source = fs::read_to_string(&fixture_path)
        .unwrap_or_else(|_| panic!("read {}", fixture_path.display()));

    let preview =
        lsp_rename_preview_v19(&source, 19, "helper", "helper_v2").expect("rename preview parse");
    assert!(
        preview.replaced_count >= 2,
        "rename must touch at least definition + one reference"
    );
    assert!(
        preview
            .edits
            .iter()
            .all(|edit| edit.new_name == "helper_v2"),
        "all rename edits must target helper_v2"
    );

    let report = json!({
        "schema": "ocp.w19.lsp.navigation_report.v1",
        "status": "PASS",
        "phase": "rename",
        "rename_preview": preview,
        "run_manifest_ref": "target/ocp/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report("lsp/navigation_report.json", &report);
}
