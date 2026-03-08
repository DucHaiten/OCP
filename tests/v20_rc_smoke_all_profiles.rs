use serde_json::json;

#[path = "v20_gate_g_common.rs"]
mod v20g;

#[test]
fn v20_rc_smoke_all_profiles() {
    v20g::ensure_run_manifest();
    let fixture = v20g::ensure_release_fixture_v20();
    assert!(
        fixture.manifest_path.exists(),
        "release manifest must exist for RC smoke"
    );
    assert!(
        fixture.manifest_sig_path.exists(),
        "release manifest signature must exist for RC smoke"
    );

    let final_scope = v20g::final_audit_scope();
    let required_profiles = final_scope
        .get("scope")
        .and_then(|scope| scope.get("supported_profiles"))
        .and_then(serde_json::Value::as_array)
        .expect("final_audit_scope.scope.supported_profiles")
        .iter()
        .map(|item| item.as_str().expect("profile string").to_string())
        .collect::<Vec<String>>();

    let aggregate_path = v20g::repo_root()
        .join("target")
        .join("ocp")
        .join("w20")
        .join("regression")
        .join("cross_platform_signature_aggregate_report.json");
    let aggregate = v20g::read_json(&aggregate_path);
    assert_eq!(
        aggregate
            .get("status")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(""),
        "PASS",
        "cross-platform aggregate report must be PASS"
    );
    let missing = aggregate
        .get("missing_profiles")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(
        missing.is_empty(),
        "cross-platform aggregate has missing profiles: {missing:?}"
    );

    let items = aggregate
        .get("items")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    for profile in &required_profiles {
        assert!(
            items.iter().any(|item| {
                item.get("os_profile").and_then(serde_json::Value::as_str) == Some(profile.as_str())
            }),
            "missing profile `{profile}` in aggregate report"
        );
    }

    let report = json!({
        "schema": "ocp.w20.release.rc_smoke_profiles_report.v1",
        "status": "PASS",
        "required_profiles": required_profiles,
        "aggregate_report_ref": "target/ocp/w20/regression/cross_platform_signature_aggregate_report.json",
        "run_manifest_ref": "target/ocp/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20g::run_manifest_sha256()
    });
    v20g::write_report("release/rc_smoke_profiles_report.json", &report);
}
