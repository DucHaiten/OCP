use std::process::Command;

#[test]
fn v100_cli_version_flag() {
    let cargo_bin = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let output = Command::new(cargo_bin)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg("run")
        .arg("-p")
        .arg("ocl-cli")
        .arg("--quiet")
        .arg("--")
        .arg("--version")
        .output()
        .expect("run ocl-cli --version");

    assert!(
        output.status.success(),
        "--version must exit successfully:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("ocl v1.0.0"),
        "--version output must contain public version: {stdout}"
    );
}
