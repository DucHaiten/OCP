use std::collections::BTreeMap;

#[path = "v18_gate_c_common.rs"]
mod common;

#[test]
fn v18_cli_alias_contract_doctor_and_fix_forward_one_to_one() {
    common::ensure_run_manifest();

    let root = common::temp_project_dir("cli_alias");
    common::init_demo_project(&root);
    common::write_manifest_with_fs_rule(&root, "locked_v06", "./**");
    let root_s = root.to_string_lossy().to_string();
    let envs = BTreeMap::new();

    let canonical_doctor = common::run_ocl_cli(&["perm", "doctor", &root_s], &envs);
    let alias_doctor = common::run_ocl_cli(&["doctor", &root_s], &envs);
    assert_eq!(
        canonical_doctor.status.code(),
        alias_doctor.status.code(),
        "doctor alias must forward exit code 1-1"
    );
    assert_eq!(
        String::from_utf8_lossy(&canonical_doctor.stdout),
        String::from_utf8_lossy(&alias_doctor.stdout),
        "doctor alias stdout must match canonical command"
    );

    let canonical_fix_plan = common::run_ocl_cli(&["perm", "fix", "--plan", &root_s], &envs);
    let alias_fix_plan = common::run_ocl_cli(&["fix", "--plan", &root_s], &envs);
    assert_eq!(
        canonical_fix_plan.status.code(),
        alias_fix_plan.status.code(),
        "fix alias must forward exit code 1-1"
    );
    assert_eq!(
        String::from_utf8_lossy(&canonical_fix_plan.stdout),
        String::from_utf8_lossy(&alias_fix_plan.stdout),
        "fix alias stdout must match canonical command"
    );
}
