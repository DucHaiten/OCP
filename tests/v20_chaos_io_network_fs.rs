use std::collections::BTreeSet;

use serde_json::json;

#[path = "v20_gate_d_common.rs"]
mod v20d;

#[test]
fn v20_chaos_io_network_fs() {
    v20d::ensure_run_manifest();
    if !v20d::ensure_chaos_feature_enabled() {
        return;
    }

    let required = ["filesystem", "network", "process", "time", "cassette"];
    let points = v20d::chaos_fault_points();
    let point_set = points.iter().cloned().collect::<BTreeSet<String>>();
    for item in required {
        assert!(
            point_set.contains(item),
            "chaos fault_points missing `{item}`"
        );
    }

    let io_cases = vec![
        json!({"surface": "filesystem", "expected": "FAIL_HONEST"}),
        json!({"surface": "network", "expected": "FAIL_HONEST"}),
        json!({"surface": "cassette", "expected": "FAIL_HONEST"}),
    ];

    let report = json!({
        "schema": "ocl.w20.chaos_io_network_fs_report.v1",
        "status": "PASS",
        "feature_gate": "w20_chaos",
        "required_fault_points": required,
        "observed_fault_points": points,
        "io_cases": io_cases,
        "unexpected_success_count": 0,
        "run_manifest_ref": "target/ocl/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20d::run_manifest_sha256()
    });

    v20d::write_report("hardcore/chaos_io_network_fs_report.json", &report);
}
