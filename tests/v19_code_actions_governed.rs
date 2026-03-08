use ocp_sdk::governed_code_action_ids_for_lane_v19;
use serde_json::json;

#[path = "v19_gate_a_common.rs"]
mod v19;

#[test]
fn v19_code_actions_governed_contract_sync() {
    v19::ensure_run_manifest();

    let code_actions_path = v19::contracts_root()
        .join("editor")
        .join("ocp_code_actions_contract.v1.json");
    let public_surface_path = v19::contracts_root()
        .join("editor")
        .join("editor_public_surface.v1.json");
    let contract = v19::read_json(&code_actions_path);
    let public_surface = v19::read_json(&public_surface_path);

    let ids = governed_code_action_ids_for_lane_v19("locked_v071", &contract, &public_surface)
        .expect(
            "strict lane governed actions must be valid and synced with public surface contract",
        );
    assert_eq!(ids.len(), 6, "governed action count must match contract");
    assert!(
        ids.iter()
            .any(|id| id == "ocp.codeAction.permissionFixPlan"),
        "permissionFixPlan action must exist"
    );
    assert!(
        ids.iter().any(|id| id == "ocp.codeAction.applyPatch"),
        "applyPatch action must exist"
    );

    let report = json!({
        "schema": "ocp.w19.editor.code_actions_report.v1",
        "status": "PASS",
        "phase": "governed",
        "action_ids": ids,
        "contract_path": code_actions_path.to_string_lossy().replace('\\', "/"),
        "public_surface_path": public_surface_path.to_string_lossy().replace('\\', "/"),
        "run_manifest_ref": "target/ocp/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report("editor/code_actions_report.json", &report);
}
