use ocp_sdk::{
    check_project, reset_warning_state_v1, run_project, set_warning_stderr_enabled_v1,
    take_warnings_v1,
};

#[path = "v100_cli_extension_common.rs"]
mod common;

#[test]
fn v100_cli_warning_dedup_contract() {
    let root = common::setup_legacy_oc_project("v100_warning_dedup");
    reset_warning_state_v1();
    set_warning_stderr_enabled_v1(false);

    check_project(root.as_path()).expect("check_project");
    run_project(root.as_path()).expect("run_project");

    let warnings = take_warnings_v1();
    let legacy_count = warnings
        .iter()
        .filter(|warning| warning.code == "W-LEGACY-OCP-EXTENSION")
        .count();
    assert_eq!(
        legacy_count, 1,
        "legacy warning must be deduplicated by cause within one execution flow: {warnings:?}"
    );
}
