use std::fs;
use std::process::Command;

use serde_json::json;

#[path = "v19_gate_g_common.rs"]
mod v19g;

#[test]
fn v19_editor_ci_entrypoint() {
    v19g::ensure_run_manifest();

    let config_path = v19g::repo_root().join(".cargo").join("config.toml");
    assert!(config_path.exists(), "missing {}", config_path.display());
    let config = fs::read_to_string(&config_path)
        .unwrap_or_else(|_| panic!("read {}", config_path.display()));
    assert!(
        config.contains("xtask = \"run -p xtask --\""),
        "missing cargo xtask alias"
    );

    let xtask_manifest = v19g::repo_root().join("xtask").join("Cargo.toml");
    let xtask_main = v19g::repo_root().join("xtask").join("src").join("main.rs");
    assert!(
        xtask_manifest.exists(),
        "missing {}",
        xtask_manifest.display()
    );
    assert!(xtask_main.exists(), "missing {}", xtask_main.display());

    let status = Command::new("cargo")
        .args(["xtask", "--help"])
        .current_dir(v19g::repo_root())
        .status()
        .expect("run cargo xtask --help");
    assert!(status.success(), "cargo xtask --help must succeed");

    let report = json!({
        "schema": "ocp.w19.community.editor_ci_entrypoint_report.v1",
        "status": "PASS",
        "entrypoint": "cargo xtask editor-ci",
        "checks": [
            ".cargo/config.toml alias present",
            "xtask crate present",
            "cargo xtask --help succeeds"
        ],
        "run_manifest_ref": "target/ocp/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19g::run_manifest_sha256()
    });
    v19g::write_report("community/editor_ci_entrypoint_report.json", &report);
}
