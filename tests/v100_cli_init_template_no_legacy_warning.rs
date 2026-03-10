#[path = "v100_cli_extension_common.rs"]
mod common;

#[path = "w17_gate_b_cli_common.rs"]
mod cli_common;

#[test]
fn v100_cli_init_template_no_legacy_warning() {
    let root = common::init_project_with_template("v100_init_no_legacy_warning", "tool-cli");
    let root_s = root.to_string_lossy().to_string();
    let check = cli_common::run_ocp_cli(&["check", &root_s]);
    cli_common::assert_success(&check);
    let stderr = String::from_utf8_lossy(&check.stderr);
    assert!(
        !stderr.contains("W-LEGACY-OCP-EXTENSION"),
        "canonical .ocp project must not emit legacy extension warning, got: {stderr}"
    );
}
