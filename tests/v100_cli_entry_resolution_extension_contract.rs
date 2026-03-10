use std::fs;

#[path = "v100_cli_extension_common.rs"]
mod common;

#[path = "w17_gate_b_cli_common.rs"]
mod cli_common;

#[test]
fn v100_cli_entry_resolution_uses_manifest_entry_instead_of_hardcoded_main() {
    let root = common::init_project_with_template("v100_entry_resolution", "mini-game");
    let root_s = root.to_string_lossy().to_string();

    fs::rename(
        root.join("src").join("main.ocp"),
        root.join("src").join("alt.ocp"),
    )
    .expect("rename src/main.ocp -> src/alt.ocp");
    common::set_project_entry(&root, "src/alt.ocp");

    let run = cli_common::run_ocp_cli(&["run", &root_s]);
    cli_common::assert_success(&run);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(
        !stderr.contains("missing entry source"),
        "run must resolve manifest entry path, got stderr: {stderr}"
    );
    assert!(
        !stderr.contains("src\\main.ocp") && !stderr.contains("src/main.ocp"),
        "run must not hard-require src/main.ocp after entry override, got stderr: {stderr}"
    );
}

#[test]
fn v100_cli_entry_resolution_rejects_unsupported_extension_even_when_missing_file() {
    let root = common::init_project_with_template("v100_entry_unsupported_missing", "mini-game");
    let root_s = root.to_string_lossy().to_string();
    common::set_project_entry(&root, "src/main.xyz");

    let check = cli_common::run_ocp_cli(&["check", &root_s, "--locked"]);
    cli_common::assert_failed_with(&check, "unsupported entry source extension");
    let stderr = String::from_utf8_lossy(&check.stderr);
    assert!(
        !stderr.contains("missing entry source"),
        "unsupported extension should be reported before missing file, got stderr: {stderr}"
    );
}
