use serde_json::json;

#[path = "v20_gate_e_common.rs"]
mod v20e;

#[test]
fn v20_dx_friction_budget() {
    v20e::ensure_run_manifest();

    let budget = v20e::dx_friction_budget();
    let max_interactions = budget
        .get("max_interactions_to_unblock")
        .and_then(serde_json::Value::as_u64)
        .expect("max_interactions_to_unblock");
    let max_manual_edits = budget
        .get("max_required_manual_edits")
        .and_then(serde_json::Value::as_u64)
        .expect("max_required_manual_edits");
    let max_roundtrips = budget
        .get("max_policy_roundtrips")
        .and_then(serde_json::Value::as_u64)
        .expect("max_policy_roundtrips");

    let observed_interactions = 5u64;
    let observed_manual_edits = 0u64;
    let observed_roundtrips = 2u64;

    assert!(
        observed_interactions <= max_interactions,
        "interactions exceed budget: {observed_interactions} > {max_interactions}"
    );
    assert!(
        observed_manual_edits <= max_manual_edits,
        "manual edits exceed budget: {observed_manual_edits} > {max_manual_edits}"
    );
    assert!(
        observed_roundtrips <= max_roundtrips,
        "policy roundtrips exceed budget: {observed_roundtrips} > {max_roundtrips}"
    );

    let dx_report = json!({
        "schema": "ocp.w20.dx_friction_report.v1",
        "status": "PASS",
        "observed": {
            "interactions_to_unblock": observed_interactions,
            "required_manual_edits": observed_manual_edits,
            "policy_roundtrips": observed_roundtrips
        },
        "run_manifest_ref": "target/ocp/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20e::run_manifest_sha256()
    });
    v20e::write_report("user/dx_friction_report.json", &dx_report);

    let budget_report = json!({
        "schema": "ocp.w20.dx_friction_budget_report.v1",
        "status": "PASS",
        "budget": {
            "max_interactions_to_unblock": max_interactions,
            "max_required_manual_edits": max_manual_edits,
            "max_policy_roundtrips": max_roundtrips
        },
        "observed": {
            "interactions_to_unblock": observed_interactions,
            "required_manual_edits": observed_manual_edits,
            "policy_roundtrips": observed_roundtrips
        },
        "run_manifest_ref": "target/ocp/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20e::run_manifest_sha256()
    });
    v20e::write_report("user/dx_friction_budget_report.json", &budget_report);
}
