use ocl_sdk::strict_lane_code_action_allowed_v19;
use serde_json::json;

#[path = "v19_gate_a_common.rs"]
mod v19;

#[test]
fn v19_code_actions_strict_lane_negative_blocks_bypass() {
    v19::ensure_run_manifest();

    let code_actions_path = v19::contracts_root()
        .join("editor")
        .join("ocl_code_actions_contract.v1.json");
    let contract = v19::read_json(&code_actions_path);

    let bypass_request = json!({
        "bypass_strict": true,
        "wildcard": false,
        "mode": "workspace_edit"
    });
    let direct_write_request = json!({
        "bypass_strict": false,
        "wildcard": false,
        "mode": "direct_write"
    });
    let valid_request = json!({
        "bypass_strict": false,
        "wildcard": false,
        "mode": "workspace_edit"
    });

    let denied_bypass = strict_lane_code_action_allowed_v19(
        "locked_v071",
        "ocpOcl.codeAction.applyPatch",
        &bypass_request,
        &contract,
    );
    let denied_direct_write = strict_lane_code_action_allowed_v19(
        "locked_v071",
        "ocpOcl.codeAction.applyPatch",
        &direct_write_request,
        &contract,
    );
    let allowed_valid = strict_lane_code_action_allowed_v19(
        "locked_v071",
        "ocpOcl.codeAction.applyPatch",
        &valid_request,
        &contract,
    );

    assert!(!denied_bypass, "strict lane must deny bypass request");
    assert!(
        !denied_direct_write,
        "strict lane must deny direct_write mode"
    );
    assert!(
        allowed_valid,
        "strict lane should allow governed workspace_edit request"
    );

    let report = json!({
        "schema": "ocl.w19.editor.code_actions_report.v1",
        "status": "PASS",
        "phase": "strict_lane_negative",
        "denied_bypass": denied_bypass,
        "denied_direct_write": denied_direct_write,
        "allowed_valid": allowed_valid,
        "contract_path": code_actions_path.to_string_lossy().replace('\\', "/"),
        "run_manifest_ref": "target/ocl/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report("editor/code_actions_report.json", &report);
}
