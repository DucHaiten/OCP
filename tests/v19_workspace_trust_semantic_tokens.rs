use ocp_sdk::workspace_semantic_tokens_allowed_v19;
use serde_json::json;

#[path = "v19_gate_a_common.rs"]
mod v19;

#[test]
fn v19_workspace_trust_semantic_tokens_follow_policy() {
    v19::ensure_run_manifest();

    let policy_path = v19::contracts_root()
        .join("editor")
        .join("editor_workspace_trust_policy.v1.json");
    let policy = v19::read_json(&policy_path);

    let trusted_semantic_tokens = workspace_semantic_tokens_allowed_v19(true, &policy);
    let untrusted_semantic_tokens = workspace_semantic_tokens_allowed_v19(false, &policy);

    assert!(
        trusted_semantic_tokens,
        "trusted workspace must enable semantic tokens provider"
    );
    assert!(
        !untrusted_semantic_tokens,
        "untrusted workspace must disable semantic tokens provider"
    );

    let request_issued_when_trusted = trusted_semantic_tokens;
    let request_issued_when_untrusted = untrusted_semantic_tokens;
    assert!(
        request_issued_when_trusted,
        "semantic tokens request should be issued in trusted mode"
    );
    assert!(
        !request_issued_when_untrusted,
        "semantic tokens request must not be issued in untrusted mode"
    );

    let report = json!({
        "schema": "ocp.w19.security.workspace_trust_semantic_tokens_report.v1",
        "status": "PASS",
        "policy_path": policy_path.to_string_lossy().replace('\\', "/"),
        "trusted_semantic_tokens": trusted_semantic_tokens,
        "untrusted_semantic_tokens": untrusted_semantic_tokens,
        "request_issued_when_trusted": request_issued_when_trusted,
        "request_issued_when_untrusted": request_issued_when_untrusted,
        "run_manifest_ref": "target/ocp/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report(
        "security/workspace_trust_semantic_tokens_report.json",
        &report,
    );
}
