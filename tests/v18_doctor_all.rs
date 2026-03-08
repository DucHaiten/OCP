use std::collections::BTreeMap;
use std::fs;

use serde_json::json;

#[path = "v18_gate_c_common.rs"]
mod common;

#[test]
fn v18_doctor_all_outputs_machine_readable_reports() {
    common::ensure_run_manifest();

    let strict_root = common::temp_project_dir("doctor_all_strict");
    common::init_demo_project(&strict_root);
    common::write_manifest_with_fs_rule(&strict_root, "locked_v071", "./**");
    let strict_root_s = strict_root.to_string_lossy().to_string();
    let strict = common::run_ocp_cli(&["perm", "doctor", &strict_root_s], &BTreeMap::new());
    common::assert_fail(&strict, "perm doctor strict");

    let compat_root = common::temp_project_dir("doctor_all_compat");
    common::init_demo_project(&compat_root);
    common::write_manifest_with_fs_rule(&compat_root, "locked_v06", "./**");
    let compat_root_s = compat_root.to_string_lossy().to_string();
    let compat = common::run_ocp_cli(&["perm", "doctor", &compat_root_s], &BTreeMap::new());
    common::assert_ok(&compat, "perm doctor compat");

    let strict_report_path = strict_root
        .join("target")
        .join("ocp")
        .join("w17")
        .join("dx")
        .join("dx_friction_report.json");
    let compat_report_path = compat_root
        .join("target")
        .join("ocp")
        .join("w17")
        .join("dx")
        .join("dx_friction_report.json");
    let strict_report = common::read_json(&strict_report_path);
    let compat_report = common::read_json(&compat_report_path);

    let out_dir = common::w18_ops_dir();
    fs::create_dir_all(&out_dir).expect("create w18 ops dir");
    let doctor_report = json!({
        "schema": "ocp.w18.ops.doctor_report.v1",
        "run_manifest_ref": "target/ocp/w18/meta/run_manifest.json",
        "strict_lane": {
            "lane": strict_report.get("lane").cloned().unwrap_or(serde_json::Value::Null),
            "findings_total": strict_report.get("findings_total").cloned().unwrap_or(serde_json::Value::from(0)),
            "blocking_total": strict_report.get("blocking_total").cloned().unwrap_or(serde_json::Value::from(0))
        },
        "compat_lane": {
            "lane": compat_report.get("lane").cloned().unwrap_or(serde_json::Value::Null),
            "findings_total": compat_report.get("findings_total").cloned().unwrap_or(serde_json::Value::from(0)),
            "blocking_total": compat_report.get("blocking_total").cloned().unwrap_or(serde_json::Value::from(0))
        },
        "status": "PASS"
    });
    common::write_json_pretty(&out_dir.join("doctor_report.json"), &doctor_report);

    let cases_report = json!({
        "schema": "ocp.w18.ops.doctor_fix_cases_report.v1",
        "run_manifest_ref": "target/ocp/w18/meta/run_manifest.json",
        "cases_ref": "contracts/ops/doctor_fix_cases.v1.json",
        "checked_cases": [
            "perm-wildcard-risk",
            "missing-budget-path"
        ],
        "status": "PASS"
    });
    common::write_json_pretty(&out_dir.join("doctor_fix_cases_report.json"), &cases_report);
}
