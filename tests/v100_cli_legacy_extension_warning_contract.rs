#[path = "v100_cli_extension_common.rs"]
mod common;

#[path = "w17_gate_b_cli_common.rs"]
mod cli_common;

#[test]
fn v100_cli_legacy_extension_warning_contract() {
    let root = common::setup_legacy_oc_project("v100_legacy_warning_contract");
    let root_s = root.to_string_lossy().to_string();
    let run = cli_common::run_ocp_cli(&["run", &root_s]);
    cli_common::assert_success(&run);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(
        stderr.contains("W-LEGACY-OCP-EXTENSION"),
        "legacy extension must emit warning code, got: {stderr}"
    );
    assert!(
        stderr.contains("legacy .oc source file"),
        "warning must identify legacy .oc extension, got: {stderr}"
    );
    assert!(
        stderr.contains("Rename to .ocp"),
        "warning must provide canonical extension migration hint, got: {stderr}"
    );
}
