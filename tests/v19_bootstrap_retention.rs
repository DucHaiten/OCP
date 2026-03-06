use ocl_sdk::bootstrap_retention_plan_v19;
use serde_json::json;

#[path = "v19_gate_f_common.rs"]
mod f19;

#[test]
fn v19_bootstrap_retention_policy_contract() {
    f19::ensure_run_manifest();
    let _fixture = f19::ensure_release_fixture_v19();

    let retention_path = f19::repo_root()
        .join("contracts")
        .join("editor")
        .join("editor_bootstrap_retention.v1.json");
    let retention = f19::read_json(&retention_path);
    let readonly_mode = retention
        .get("readonly_storage_mode")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    assert_eq!(readonly_mode, "fail_honest_degraded");

    let versions = vec![
        "1.0.0".to_string(),
        "1.1.0".to_string(),
        "1.2.0".to_string(),
        "1.3.0".to_string(),
        "1.4.0".to_string(),
    ];
    let active = "1.4.0";
    let (kept, removed) =
        bootstrap_retention_plan_v19(&retention, &versions, active).expect("retention plan");

    let max_versions = retention
        .get("max_versions_per_platform")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(3) as usize;
    assert!(
        kept.len() <= max_versions + 1,
        "kept set should respect retention cap"
    );
    assert!(
        kept.iter().any(|item| item == active),
        "active version must never be removed"
    );
    assert!(
        removed.iter().all(|item| item != active),
        "removed set must not include active version"
    );

    let report = json!({
        "schema": "ocl.w19.release.bootstrap_retention_report.v1",
        "status": "PASS",
        "retention_contract_path": retention_path.to_string_lossy().replace('\\', "/"),
        "active_version": active,
        "input_versions": versions,
        "kept_versions": kept,
        "removed_versions": removed,
        "readonly_storage_mode": readonly_mode,
        "run_manifest_ref": "target/ocl/w19/meta/run_manifest.json",
        "run_manifest_sha256": f19::run_manifest_sha256()
    });
    f19::write_report("release/bootstrap_retention_report.json", &report);
}
