use std::collections::BTreeMap;

mod w17_perm_common;

#[test]
fn v17_error_taxonomy_contract_for_perm_guard_is_stable() {
    let root = w17_perm_common::temp_project_dir("taxonomy_guard");
    w17_perm_common::init_demo_project(&root);
    w17_perm_common::write_manifest_with_fs_rule(&root, "locked_v071", "./**");

    let root_s = root.to_string_lossy().to_string();
    let output = w17_perm_common::run_ocp_cli(&["perm", "doctor", &root_s], &BTreeMap::new());
    assert!(!output.status.success(), "doctor must fail in strict lane");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stderr.contains("X-PERM-RUBBERSTAMP-GUARD"),
        "code must be stable, got: {stderr}"
    );
    assert!(
        stderr.contains("RC-PERM-WILDCARD-DENIED"),
        "reason must be stable, got: {stderr}"
    );
    assert!(
        stderr.contains("X-PERMISSION-REVIEW-REQUIRED"),
        "alias must be stable, got: {stderr}"
    );
    assert!(
        stderr.contains("severity=`error`"),
        "severity must stay strict, got: {stderr}"
    );
    assert!(stderr.contains("hint=`"), "hint must exist, got: {stderr}");
}
