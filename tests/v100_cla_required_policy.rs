use serde_json::{json, Value as JsonValue};

#[path = "v100_gate_g_common.rs"]
mod v100g;

#[test]
fn v100_cla_required_policy() {
    v100g::ensure_run_manifest();

    let licensing_model = v100g::licensing_model();
    let community_policy = v100g::community_policy();
    let contributing = v100g::read_text(&v100g::repo_root().join("CONTRIBUTING.md"));

    assert!(licensing_model
        .get("cla_required")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false));
    assert!(community_policy
        .get("reject_contribution_without_cla")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false));
    assert!(
        contributing.contains("CLA")
            && contributing.contains("license_grant")
            && contributing.contains("AGPL-3.0-only+commercial"),
        "CONTRIBUTING.md must explain required CLA flow and outbound model"
    );

    let report = json!({
        "schema": "ocl.w100.business.cla_policy_contract_report.v1",
        "status": "PASS",
        "cla_required": true,
        "reject_contribution_without_cla": true,
        "contributing_verified": "CONTRIBUTING.md",
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100g::run_manifest_sha256()
    });
    v100g::write_report("business/cla_policy_contract_report.json", &report);
}
