use std::collections::BTreeMap;
use std::fs;

use serde_json::json;

#[path = "v18_gate_c_common.rs"]
mod common;

#[test]
fn v18_perm_rubberstamp_guard_strict_vs_compat_contract() {
    common::ensure_run_manifest();

    let strict_root = common::temp_project_dir("rubber_strict");
    common::init_demo_project(&strict_root);
    common::write_manifest_with_fs_rule(&strict_root, "locked_v071", "./**");
    let strict_root_s = strict_root.to_string_lossy().to_string();
    let strict = common::run_ocp_cli(&["perm", "doctor", &strict_root_s], &BTreeMap::new());
    common::assert_fail(&strict, "perm doctor strict lane");

    let compat_root = common::temp_project_dir("rubber_compat");
    common::init_demo_project(&compat_root);
    common::write_manifest_with_fs_rule(&compat_root, "locked_v06", "./**");
    let compat_root_s = compat_root.to_string_lossy().to_string();
    let compat = common::run_ocp_cli(&["perm", "doctor", &compat_root_s], &BTreeMap::new());
    common::assert_ok(&compat, "perm doctor compat lane");

    let compat_report = compat_root
        .join("target")
        .join("ocp")
        .join("w17")
        .join("dx")
        .join("dx_friction_report.json");
    let report_json = common::read_json(&compat_report);
    let blocking_total = report_json
        .get("blocking_total")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(u64::MAX);
    assert_eq!(
        blocking_total, 0,
        "locked_v06 must not block wildcard by default"
    );

    let out_dir = common::w18_ops_dir();
    fs::create_dir_all(&out_dir).expect("create w18 ops dir");
    let report = json!({
        "schema": "ocp.w18.ops.perm_rubberstamp_guard_report.v1",
        "run_manifest_ref": "target/ocp/w18/meta/run_manifest.json",
        "strict_lane_status": "blocked",
        "compat_lane_status": "warn_only",
        "status": "PASS"
    });
    common::write_json_pretty(&out_dir.join("perm_rubberstamp_guard_report.json"), &report);
}
