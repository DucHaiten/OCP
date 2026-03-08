use std::collections::BTreeMap;

mod w17_perm_common;

#[test]
fn v17_perm_doctor_blocks_wildcard_in_locked_lane_and_writes_report() {
    let root = w17_perm_common::temp_project_dir("doctor_locked");
    w17_perm_common::init_demo_project(&root);
    w17_perm_common::write_manifest_with_fs_rule(&root, "locked_v071", "./**");

    let root_s = root.to_string_lossy().to_string();
    let output = w17_perm_common::run_ocp_cli(&["perm", "doctor", &root_s], &BTreeMap::new());
    assert!(
        !output.status.success(),
        "perm doctor must fail in locked_v071"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("X-PERM-RUBBERSTAMP-GUARD"),
        "stderr must include guard code, got: {stderr}"
    );
    assert!(
        stderr.contains("RC-PERM-WILDCARD-DENIED"),
        "stderr must include reason code, got: {stderr}"
    );
    assert!(
        stderr.contains("X-PERMISSION-REVIEW-REQUIRED"),
        "stderr must include alias, got: {stderr}"
    );

    let report = root
        .join("target")
        .join("ocp")
        .join("w17")
        .join("dx")
        .join("dx_friction_report.json");
    assert!(report.exists(), "doctor report must be written");
}
