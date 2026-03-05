use std::path::PathBuf;
use std::process::{Command, Output};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn run_ocl_cli(args: &[&str]) -> Output {
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

#[test]
fn upgrade_check_negative_invalid_runtime_mode_fails_with_hint() {
    let output = run_ocl_cli(&[
        "upgrade-check",
        "projects/ocp-ocl/conformance/fixtures/core-exec-pass",
        "--manifest",
        "projects/ocp-ocl/conformance/conformance.v5.toml",
        "--runtime",
        "invalid-runtime",
        "--json",
    ]);
    assert!(
        !output.status.success(),
        "upgrade-check should fail for invalid runtime mode\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("expected deterministic|throughput"),
        "stderr must include runtime hint, got: {stderr}"
    );
}

#[test]
fn upgrade_check_negative_invalid_engine_mode_fails_with_hint() {
    let output = run_ocl_cli(&[
        "upgrade-check",
        "projects/ocp-ocl/conformance/fixtures/core-exec-pass",
        "--manifest",
        "projects/ocp-ocl/conformance/conformance.v5.toml",
        "--engine",
        "invalid-engine",
        "--json",
    ]);
    assert!(
        !output.status.success(),
        "upgrade-check should fail for invalid engine mode\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("expected interpreter|bytecode|dual"),
        "stderr must include engine hint, got: {stderr}"
    );
}
