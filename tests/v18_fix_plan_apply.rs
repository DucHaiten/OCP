use std::collections::BTreeMap;
use std::fs;

use serde_json::json;

#[path = "v18_gate_c_common.rs"]
mod common;

#[test]
fn v18_fix_plan_apply_generates_plan_patch_report_and_approval_record() {
    common::ensure_run_manifest();

    let root = common::temp_project_dir("fix_plan_apply");
    common::init_demo_project(&root);
    common::write_manifest_with_fs_rule(&root, "locked_v071", "./**");
    let root_s = root.to_string_lossy().to_string();
    let empty = BTreeMap::new();

    let plan = common::run_ocl_cli(&["perm", "fix", "--plan", &root_s], &empty);
    common::assert_ok(&plan, "perm fix --plan");
    let apply = common::run_ocl_cli(
        &[
            "perm",
            "fix",
            "--apply",
            &root_s,
            "--ack-risk",
            "--justification",
            "narrow wildcard permissions before release",
            "--by",
            "ci-bot",
            "--date",
            "2026-03-06",
        ],
        &empty,
    );
    common::assert_ok(&apply, "perm fix --apply");

    let dx_dir = root.join("target").join("ocl").join("w17").join("dx");
    let plan_file = dx_dir.join("permission_fix_plan.json");
    let patch_file = dx_dir.join("permission_fix.patch.toml");
    let safety_file = dx_dir.join("permission_fix_safety_report.json");
    let approval_file = root.join("permissions.approval.toml");
    assert!(plan_file.exists(), "missing permission_fix_plan.json");
    assert!(patch_file.exists(), "missing permission_fix.patch.toml");
    assert!(
        safety_file.exists(),
        "missing permission_fix_safety_report.json"
    );
    assert!(approval_file.exists(), "missing permissions.approval.toml");

    let out_dir = common::w18_ops_dir();
    fs::create_dir_all(&out_dir).expect("create w18 ops dir");
    let report = json!({
        "schema": "ocl.w18.ops.fix_plan_report.v1",
        "run_manifest_ref": "target/ocl/w18/meta/run_manifest.json",
        "project_root": root.to_string_lossy().replace('\\', "/"),
        "plan_path": plan_file.to_string_lossy().replace('\\', "/"),
        "patch_path": patch_file.to_string_lossy().replace('\\', "/"),
        "safety_report_path": safety_file.to_string_lossy().replace('\\', "/"),
        "approval_path": approval_file.to_string_lossy().replace('\\', "/"),
        "status": "PASS"
    });
    common::write_json_pretty(&out_dir.join("fix_plan_report.json"), &report);
}
