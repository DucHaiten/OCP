use std::collections::BTreeMap;

mod w17_perm_common;

#[test]
fn v17_perm_fix_apply_requires_ack_and_justification() {
    let root = w17_perm_common::temp_project_dir("fix_negative");
    w17_perm_common::init_demo_project(&root);
    w17_perm_common::write_manifest_with_fs_rule(&root, "locked_v071", "./**");

    let root_s = root.to_string_lossy().to_string();
    let output = w17_perm_common::run_ocl_cli(
        &[
            "perm",
            "fix",
            "--apply",
            &root_s,
            "--justification",
            "test missing ack",
            "--by",
            "ci-bot",
            "--date",
            "2026-03-06",
        ],
        &BTreeMap::new(),
    );
    assert!(
        !output.status.success(),
        "perm fix --apply must fail without --ack-risk"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("X-PERMISSION-RISK-ACK-REQUIRED"),
        "stderr must include ack guard code, got: {stderr}"
    );
    assert!(
        stderr.contains("RC-PERMISSION-RISK-ACK-REQUIRED"),
        "stderr must include ack reason code, got: {stderr}"
    );
}
