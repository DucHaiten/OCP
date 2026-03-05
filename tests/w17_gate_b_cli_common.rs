#![allow(dead_code)]

use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn temp_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocl_v17_gate_b_{tag}_{stamp}"))
}

pub fn run_ocl_cli(args: &[&str]) -> Output {
    let cargo_bin = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    Command::new(cargo_bin)
        .current_dir(repo_root())
        .arg("run")
        .arg("-p")
        .arg("ocl-cli")
        .arg("--quiet")
        .arg("--")
        .args(args)
        .output()
        .expect("run ocl-cli")
}

pub fn assert_success(output: &Output) -> String {
    assert!(
        output.status.success(),
        "command failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

pub fn assert_failed_with(output: &Output, needle: &str) {
    assert!(
        !output.status.success(),
        "command should fail:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(needle),
        "expected `{needle}` in stderr, got: {stderr}"
    );
}

pub fn ensure_artifact_layout(root: &PathBuf) -> PathBuf {
    let artifact = root.join(".ocl_artifacts").join("run_demo");
    std::fs::create_dir_all(artifact.join("cassette")).expect("create cassette dir");
    artifact
}
