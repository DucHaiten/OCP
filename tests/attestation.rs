use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use ocl_sdk::{init_project, resolve_deps_v3, sync_deps_lock_v1};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocl_v15_attestation_{tag}_{stamp}"))
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

fn write_manifest(root: &Path) {
    let manifest = concat!(
        "[package]\n",
        "name = \"attestation_demo\"\n",
        "version = \"0.1.0\"\n\n",
        "[project]\n",
        "lane = \"locked_v071\"\n\n",
        "[dependencies]\n",
        "std = \"0.1.0\"\n\n",
        "[permissions.package]\n",
        "allow = [\"*\"]\n",
        "deny = [\"std.net.poll\"]\n",
    );
    fs::write(root.join("Ocl.toml"), manifest).expect("write Ocl.toml");
}

fn setup_project(root: &Path) {
    init_project(root).expect("init");
    write_manifest(root);
    sync_deps_lock_v1(root).expect("sync deps lock v1");
    resolve_deps_v3(root, false).expect("resolve deps lock v3");
}

#[test]
fn v15_build_attest_and_verify_repro_pass() {
    let root = temp_project_dir("verify_repro_ok");
    setup_project(&root);
    let root_s = root.to_string_lossy().to_string();
    let envs = BTreeMap::new();

    let build = run_ocl_cli(&["build", &root_s, "--source-only", "--attest"], &envs);
    assert_ok(&build, "build --attest");

    let artifact_dir = root.join("target").join("ocl").join("attestation");
    assert!(
        artifact_dir.join("build_manifest.json").exists(),
        "missing build_manifest.json"
    );
    assert!(
        artifact_dir.join("build_manifest.sig").exists(),
        "missing build_manifest.sig"
    );
    let artifact_dir_s = artifact_dir.to_string_lossy().to_string();

    let verify_attest = run_ocl_cli(&["verify", "--attest", &artifact_dir_s], &envs);
    assert_ok(&verify_attest, "verify --attest");

    let verify_repro = run_ocl_cli(&["verify", "--repro", &artifact_dir_s], &envs);
    assert_ok(&verify_repro, "verify --repro");
}

#[test]
fn v15_verify_attest_fails_when_manifest_tampered() {
    let root = temp_project_dir("tamper_manifest");
    setup_project(&root);
    let root_s = root.to_string_lossy().to_string();
    let envs = BTreeMap::new();

    let build = run_ocl_cli(&["build", &root_s, "--source-only", "--attest"], &envs);
    assert_ok(&build, "build --attest");

    let artifact_dir = root.join("target").join("ocl").join("attestation");
    let manifest_path = artifact_dir.join("build_manifest.json");
    let raw_manifest = fs::read_to_string(&manifest_path).expect("read manifest");
    let tampered_manifest =
        raw_manifest.replace("\"lane\":\"locked_v071\"", "\"lane\":\"locked_v06\"");
    assert_ne!(
        raw_manifest, tampered_manifest,
        "tamper must change manifest content"
    );
    fs::write(&manifest_path, tampered_manifest).expect("write tampered manifest");

    let artifact_dir_s = artifact_dir.to_string_lossy().to_string();
    let verify_attest = run_ocl_cli(&["verify", "--attest", &artifact_dir_s], &envs);
    assert!(
        !verify_attest.status.success(),
        "verify --attest must fail on tampered manifest"
    );
    let stderr = String::from_utf8_lossy(&verify_attest.stderr);
    assert!(
        stderr.contains("X-ATTEST-SIGNATURE-MISMATCH"),
        "unexpected stderr: {stderr}"
    );
}
