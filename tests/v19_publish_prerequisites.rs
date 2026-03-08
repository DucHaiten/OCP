use std::path::Path;

use ocp_sdk::{required_publish_files_v19, required_publish_metadata_fields_v19};
use serde_json::json;

#[path = "v19_gate_f_common.rs"]
mod f19;

fn has_dotted_field(root: &serde_json::Value, dotted: &str) -> bool {
    let mut current = root;
    for part in dotted.split('.') {
        let Some(next) = current.get(part) else {
            return false;
        };
        current = next;
    }
    true
}

#[test]
fn v19_publish_prerequisites_are_present() {
    f19::ensure_run_manifest();
    let _fixture = f19::ensure_release_fixture_v19();

    let prereq_path = f19::repo_root()
        .join("contracts")
        .join("editor")
        .join("editor_publish_prerequisites.v1.json");
    let prereq = f19::read_json(&prereq_path);
    let required_files = required_publish_files_v19(&prereq).expect("required files");
    let required_fields =
        required_publish_metadata_fields_v19(&prereq).expect("required metadata fields");

    let mut missing_files = Vec::<String>::new();
    for rel in &required_files {
        let abs = f19::repo_root().join(Path::new(rel));
        if !abs.exists() {
            missing_files.push(rel.clone());
        }
    }
    assert!(
        missing_files.is_empty(),
        "missing publish prerequisite files"
    );

    let package_path = f19::repo_root()
        .join("editor")
        .join("vscode")
        .join("ocp")
        .join("package.json");
    let package = f19::read_json(&package_path);
    let mut missing_fields = Vec::<String>::new();
    for field in &required_fields {
        if !has_dotted_field(&package, field) {
            missing_fields.push(field.clone());
        }
    }
    assert!(missing_fields.is_empty(), "missing package metadata fields");

    let report = json!({
        "schema": "ocp.w19.release.publish_prerequisites_report.v1",
        "status": "PASS",
        "prereq_contract_path": prereq_path.to_string_lossy().replace('\\', "/"),
        "checked_files_count": required_files.len(),
        "checked_metadata_fields_count": required_fields.len(),
        "run_manifest_ref": "target/ocp/w19/meta/run_manifest.json",
        "run_manifest_sha256": f19::run_manifest_sha256()
    });
    f19::write_report("release/publish_prerequisites_report.json", &report);
}
