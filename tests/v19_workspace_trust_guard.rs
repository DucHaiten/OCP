use ocp_sdk::workspace_runtime_features_allowed_v19;
use serde_json::json;

#[path = "v19_gate_a_common.rs"]
mod v19;

#[test]
fn v19_workspace_trust_guard_blocks_runtime_features_when_untrusted() {
    v19::ensure_run_manifest();

    let policy_path = v19::contracts_root()
        .join("editor")
        .join("editor_workspace_trust_policy.v1.json");
    let policy = v19::read_json(&policy_path);

    let trusted_allowed = workspace_runtime_features_allowed_v19(true, &policy);
    let untrusted_allowed = workspace_runtime_features_allowed_v19(false, &policy);

    assert!(
        trusted_allowed,
        "trusted workspace must allow runtime feature activation"
    );
    assert!(
        !untrusted_allowed,
        "untrusted workspace must block runtime feature activation"
    );

    let report = json!({
        "schema": "ocp.w19.security.workspace_trust_report.v1",
        "status": "PASS",
        "policy_path": policy_path.to_string_lossy().replace('\\', "/"),
        "trusted_runtime_allowed": trusted_allowed,
        "untrusted_runtime_allowed": untrusted_allowed,
        "run_manifest_ref": "target/ocp/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report("security/workspace_trust_report.json", &report);
}
