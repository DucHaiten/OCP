use serde_json::json;

#[path = "v19_gate_g_common.rs"]
mod v19g;

#[test]
fn v19_release_positioning_guard() {
    v19g::ensure_run_manifest();

    let harness = v19g::read_json("contracts/editor/editor_ci_harness.v1.json");
    let publish_channels = v19g::read_json("contracts/editor/editor_publish_channels.v1.json");
    let perf_budget = v19g::read_json("contracts/editor/editor_perf_budget.v1.json");
    let version_matrix = v19g::read_json("contracts/editor/ocl_version_compat_matrix.v1.json");

    let profiles = harness
        .get("os_profiles")
        .and_then(|v| v.as_array())
        .expect("os_profiles")
        .iter()
        .map(|item| item.as_str().expect("profile string").to_string())
        .collect::<Vec<String>>();
    assert_eq!(
        profiles.len(),
        3,
        "release profile must remain locked to 3 OS targets"
    );

    let channels = publish_channels
        .get("channels")
        .and_then(|v| v.as_array())
        .expect("channels")
        .iter()
        .map(|item| item.as_str().expect("channel str").to_string())
        .collect::<Vec<String>>();
    assert!(
        channels.iter().any(|c| c == "vscode_marketplace"),
        "missing vscode_marketplace channel"
    );
    assert!(
        channels.iter().any(|c| c == "open_vsx"),
        "missing open_vsx channel"
    );

    let thresholds = perf_budget
        .get("thresholds")
        .and_then(|v| v.as_object())
        .expect("perf thresholds");
    for forbidden in ["wallclock_ms", "cpu_ms", "latency_ms"] {
        assert!(
            !thresholds.contains_key(forbidden),
            "perf positioning must not use wallclock metric `{forbidden}` as pass criterion"
        );
    }

    let compatibility_entries = version_matrix
        .get("rows")
        .and_then(|v| v.as_array())
        .map(|v| v.len())
        .unwrap_or(0usize);
    assert!(
        compatibility_entries > 0,
        "version compatibility matrix must not be empty"
    );

    let report = json!({
        "schema": "ocl.w19.rc.editor_release_readiness_report.v1",
        "status": "PASS",
        "positioning_guard": {
            "supported_profiles": profiles,
            "publish_channels": channels,
            "perf_criterion": "deterministic_work_units_only",
            "unsupported_claims": []
        },
        "compatibility_entries": compatibility_entries,
        "run_manifest_ref": "target/ocl/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19g::run_manifest_sha256()
    });
    v19g::write_report("rc/editor_release_readiness_report.json", &report);
}
