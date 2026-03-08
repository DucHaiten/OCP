use std::fs;

use ocp_sdk::{lsp_multiroot_definition_locations_v19, multiroot_sorted_roots_v19};
use serde_json::json;

#[path = "v19_gate_a_common.rs"]
mod v19;

fn normalize(value: &str) -> String {
    value.replace('\\', "/").to_lowercase()
}

#[test]
fn v19_multiroot_workspace_policy_is_deterministic() {
    v19::ensure_run_manifest();

    let policy_path = v19::contracts_root()
        .join("editor")
        .join("editor_multiroot_policy.v1.json");
    let policy = v19::read_json(&policy_path);

    let root_z = v19::repo_root()
        .join("tests")
        .join("fixtures")
        .join("v19")
        .join("multiroot")
        .join("root_z");
    let root_a = v19::repo_root()
        .join("tests")
        .join("fixtures")
        .join("v19")
        .join("multiroot")
        .join("root_a");
    let root_z_s = root_z.to_string_lossy().to_string();
    let root_a_s = root_a.to_string_lossy().to_string();

    let roots = [root_z_s.clone(), root_a_s.clone()];
    let sorted = multiroot_sorted_roots_v19(&roots, &policy);
    assert_eq!(
        sorted.len(),
        2,
        "sorted roots must keep both workspace folders"
    );
    assert_eq!(
        normalize(&sorted[0]),
        normalize(&root_a_s),
        "canonical root ordering must sort root_a before root_z"
    );

    let source_z = fs::read_to_string(root_z.join("shared.ocp")).expect("read root_z");
    let source_a = fs::read_to_string(root_a.join("shared.ocp")).expect("read root_a");
    let files = vec![
        (root_z_s.clone(), 1901_u32, source_z),
        (root_a_s.clone(), 1902_u32, source_a),
    ];
    let defs =
        lsp_multiroot_definition_locations_v19(&files, "shared", &policy).expect("multiroot parse");
    assert_eq!(defs.len(), 2, "shared must be defined in both roots");
    assert_eq!(
        normalize(defs[0].root.as_deref().unwrap_or_default()),
        normalize(&root_a_s),
        "definition ordering must follow multiroot policy"
    );

    let report = json!({
        "schema": "ocp.w19.lsp.multiroot_workspace_report.v1",
        "status": "PASS",
        "policy_path": policy_path.to_string_lossy().replace('\\', "/"),
        "sorted_roots": sorted.iter().map(|item| item.replace('\\', "/")).collect::<Vec<String>>(),
        "definition_count": defs.len(),
        "definitions": defs,
        "run_manifest_ref": "target/ocp/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report("lsp/multiroot_workspace_report.json", &report);
}
