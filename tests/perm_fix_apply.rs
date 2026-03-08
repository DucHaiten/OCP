use std::collections::BTreeMap;

mod w17_perm_common;

#[test]
fn v17_perm_fix_apply_writes_patch_and_approval_without_bypassing_policy() {
    let root = w17_perm_common::temp_project_dir("fix_apply");
    w17_perm_common::init_demo_project(&root);
    w17_perm_common::write_manifest_with_fs_rule(&root, "locked_v071", "./**");

    let root_s = root.to_string_lossy().to_string();
    let empty_env = BTreeMap::new();

    let plan = w17_perm_common::run_ocp_cli(&["perm", "fix", "--plan", &root_s], &empty_env);
    w17_perm_common::assert_ok(&plan, "perm fix --plan");

    let apply = w17_perm_common::run_ocp_cli(
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
        &empty_env,
    );
    w17_perm_common::assert_ok(&apply, "perm fix --apply");

    let dx_dir = root.join("target").join("ocp").join("w17").join("dx");
    assert!(
        dx_dir.join("permission_fix_plan.json").exists(),
        "missing permission_fix_plan.json"
    );
    assert!(
        dx_dir.join("permission_fix.patch.toml").exists(),
        "missing permission_fix.patch.toml"
    );
    assert!(
        dx_dir.join("permission_fix_safety_report.json").exists(),
        "missing permission_fix_safety_report.json"
    );
    assert!(
        root.join("permissions.approval.toml").exists(),
        "missing permissions.approval.toml"
    );

    let doctor_after = w17_perm_common::run_ocp_cli(&["perm", "doctor", &root_s], &empty_env);
    assert!(
        !doctor_after.status.success(),
        "perm fix apply must not bypass strict lane guard while manifest still has wildcard"
    );
}
