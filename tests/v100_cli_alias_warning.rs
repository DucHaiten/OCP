use std::process::Command;

fn run_cli(bin: &str) -> std::process::Output {
    let cargo_bin = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    Command::new(cargo_bin)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg("run")
        .arg("-p")
        .arg("ocp-cli")
        .arg("--bin")
        .arg(bin)
        .arg("--quiet")
        .arg("--")
        .arg("--version")
        .output()
        .expect("run cli --version")
}

#[test]
fn v100_cli_alias_warning_for_legacy_ocp_binary() {
    let legacy = run_cli("ocp");
    assert!(
        legacy.status.success(),
        "legacy alias must run:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&legacy.stdout),
        String::from_utf8_lossy(&legacy.stderr)
    );
    let legacy_stdout = String::from_utf8_lossy(&legacy.stdout);
    let legacy_stderr = String::from_utf8_lossy(&legacy.stderr);
    assert!(
        legacy_stdout.contains("ocp v1.0.0"),
        "legacy alias must report ocp version: {legacy_stdout}"
    );
    assert!(
        legacy_stderr.contains("W-CLI-ALIAS-DEPRECATED"),
        "legacy alias must emit deprecation warning: {legacy_stderr}"
    );

    let primary = run_cli("ocp");
    assert!(
        primary.status.success(),
        "primary cli must run:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&primary.stdout),
        String::from_utf8_lossy(&primary.stderr)
    );
    let primary_stdout = String::from_utf8_lossy(&primary.stdout);
    let primary_stderr = String::from_utf8_lossy(&primary.stderr);
    assert!(
        primary_stdout.contains("ocp v1.0.0"),
        "primary cli must report ocp version: {primary_stdout}"
    );
    assert!(
        !primary_stderr.contains("W-CLI-ALIAS-DEPRECATED"),
        "primary cli must not emit alias warning: {primary_stderr}"
    );
}
