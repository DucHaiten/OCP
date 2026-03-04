use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocl_v10_override_guardrails_{tag}_{stamp}"))
}

fn run_ocl_cli(args: &[&str], envs: &BTreeMap<&str, &str>) -> Output {
    let cargo_bin = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut cmd = Command::new(cargo_bin);
    cmd.current_dir(repo_root())
        .arg("run")
        .arg("-p")
        .arg("ocl-cli")
        .arg("--quiet")
        .arg("--")
        .args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("run ocl-cli")
}

fn assert_ok(output: &Output, step: &str) {
    assert!(
        output.status.success(),
        "{step} failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn write_manifest_with_override(root: &Path, allow_overrides: bool) {
    let mut manifest = String::new();
    manifest.push_str(concat!(
        "[package]\n",
        "name = \"override_guardrails_demo\"\n",
        "version = \"0.1.0\"\n\n",
        "[project]\n",
        "lane = \"locked_v071\"\n\n",
        "[targets]\n",
        "default = \"main\"\n\n",
        "[dependencies]\n",
        "std = \"0.1.0\"\n\n",
        "[permissions.package]\n",
        "allow = [\"std.*\"]\n",
        "deny = []\n\n",
        "[permission_overrides.widgets]\n",
        "std_proc.allow_bins = [\"python\"]\n",
    ));
    if allow_overrides {
        manifest.push_str("\n[security]\nallow_overrides = true\n");
    }
    fs::write(root.join("Ocl.toml"), manifest).expect("write Ocl.toml");
}

#[test]
fn v10_cli_deps_verify_enforces_override_guardrails() {
    let root = temp_project_dir("override_guardrails");
    let root_s = root.to_string_lossy().to_string();
    let empty_env = BTreeMap::new();

    let init = run_ocl_cli(&["init", &root_s], &empty_env);
    assert_ok(&init, "init");

    write_manifest_with_override(&root, false);
    let resolve = run_ocl_cli(&["deps", "resolve", &root_s], &empty_env);
    assert_ok(&resolve, "deps resolve");

    let verify_disabled = run_ocl_cli(&["deps", "verify", &root_s], &empty_env);
    assert!(
        !verify_disabled.status.success(),
        "deps verify must fail when override section exists but allow_overrides=false"
    );
    let stderr_disabled = String::from_utf8_lossy(&verify_disabled.stderr);
    assert!(
        stderr_disabled.contains("X-PERMISSION-OVERRIDE-DISABLED"),
        "unexpected stderr: {stderr_disabled}"
    );

    write_manifest_with_override(&root, true);
    let verify_missing_env = run_ocl_cli(&["deps", "verify", &root_s], &empty_env);
    assert!(
        !verify_missing_env.status.success(),
        "deps verify must fail when env gate is missing"
    );
    let stderr_missing_env = String::from_utf8_lossy(&verify_missing_env.stderr);
    assert!(
        stderr_missing_env.contains("V-PERMISSION-OVERRIDE-ENV-REQUIRED"),
        "unexpected stderr: {stderr_missing_env}"
    );

    let mut override_env = BTreeMap::new();
    override_env.insert("OCL_ALLOW_OVERRIDES", "1");
    let verify_pass = run_ocl_cli(&["deps", "verify", &root_s], &override_env);
    assert_ok(&verify_pass, "deps verify with override env");
}
