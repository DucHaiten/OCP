use std::collections::BTreeMap;

#[path = "v18_gate_c_common.rs"]
mod common;

#[test]
fn v18_fix_strict_lane_negative_requires_ack_and_justification() {
    common::ensure_run_manifest();

    let root = common::temp_project_dir("fix_strict_negative");
    common::init_demo_project(&root);
    common::write_manifest_with_fs_rule(&root, "locked_v071", "./**");
    let root_s = root.to_string_lossy().to_string();

    let out = common::run_ocl_cli(
        &[
            "perm",
            "fix",
            "--apply",
            &root_s,
            "--justification",
            "missing ack",
            "--by",
            "ci-bot",
            "--date",
            "2026-03-06",
        ],
        &BTreeMap::new(),
    );
    common::assert_fail(&out, "perm fix --apply without --ack-risk");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("X-PERMISSION-RISK-ACK-REQUIRED"),
        "stderr must include ack guard code, got: {stderr}"
    );
    assert!(
        stderr.contains("RC-PERMISSION-RISK-ACK-REQUIRED"),
        "stderr must include ack reason code, got: {stderr}"
    );
}
