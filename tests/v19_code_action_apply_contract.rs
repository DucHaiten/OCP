use ocl_sdk::{code_action_apply_policy_v19, validate_code_action_apply_request_v19};
use serde_json::json;

#[path = "v19_gate_a_common.rs"]
mod v19;

#[test]
fn v19_code_action_apply_contract_is_enforced() {
    v19::ensure_run_manifest();

    let contract_path = v19::contracts_root()
        .join("editor")
        .join("editor_code_action_apply_policy.v1.json");
    let contract = v19::read_json(&contract_path);

    let policy = code_action_apply_policy_v19(&contract).expect("parse apply policy contract");
    assert_eq!(policy.patch_format, "workspace_edit_v1");
    assert_eq!(policy.apply_mode, "workspace_edit");
    assert_eq!(policy.record_creation_point, "apply");
    assert!(!policy.allow_cli_apply_fallback);

    let valid_request = json!({
        "patch_format": "workspace_edit_v1",
        "apply_mode": "workspace_edit",
        "record_created": true
    });
    let invalid_request = json!({
        "patch_format": "workspace_edit_v1",
        "apply_mode": "workspace_edit",
        "record_created": false
    });

    let valid = validate_code_action_apply_request_v19("locked_v071", &valid_request, &policy);
    let invalid = validate_code_action_apply_request_v19("locked_v071", &invalid_request, &policy);
    assert!(valid, "valid apply request must pass");
    assert!(!invalid, "missing record at apply point must fail");

    let report = json!({
        "schema": "ocl.w19.editor.code_action_apply_contract_report.v1",
        "status": "PASS",
        "contract_path": contract_path.to_string_lossy().replace('\\', "/"),
        "policy": policy,
        "valid_request_pass": valid,
        "invalid_request_pass": invalid,
        "run_manifest_ref": "target/ocl/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report("editor/code_action_apply_contract_report.json", &report);
}
