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
    std::env::temp_dir().join(format!("ocl_v10_cli_deps_e2e_{tag}_{stamp}"))
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

fn read_non_builtin_signer(lock_path: &Path) -> String {
    let raw = fs::read_to_string(lock_path).expect("read deps.lock.v3");
    for line in raw.lines() {
        let Some(payload) = line.strip_prefix("dep=") else {
            continue;
        };
        let parts: Vec<&str> = payload.split('|').collect();
        if parts.len() == 11 && parts[3] != "builtin" {
            return parts[7].to_string();
        }
    }
    panic!("non-builtin signer not found in deps.lock.v3");
}

fn first_oclpkg(root: &Path) -> PathBuf {
    let pkg_dir = root.join(".oclpkg");
    let entries = fs::read_dir(&pkg_dir).expect("read .oclpkg");
    for entry in entries {
        let path = entry.expect("entry").path();
        if path.extension().and_then(|s| s.to_str()) == Some("oclpkg") {
            return path;
        }
    }
    panic!("no .oclpkg artifact found in {}", pkg_dir.display());
}

#[test]
fn v10_cli_deps_and_pack_commands_work_end_to_end() {
    let root = temp_project_dir("full_flow");
    let root_s = root.to_string_lossy().to_string();
    let empty_env = BTreeMap::new();

    let init = run_ocl_cli(
        &["init", &root_s, "--template", "dep-permission"],
        &empty_env,
    );
    assert_ok(&init, "init dep-permission");
    assert!(root
        .join("deps")
        .join("widgets")
        .join("package.oclp")
        .exists());

    let resolve = run_ocl_cli(&["deps", "resolve", &root_s], &empty_env);
    assert_ok(&resolve, "deps resolve");
    assert!(root.join("deps.lock.v3").exists(), "missing deps.lock.v3");
    assert!(root.join("ocl.lock").exists(), "missing ocl.lock");

    let signer = read_non_builtin_signer(&root.join("deps.lock.v3"));
    let trust = format!("[trusted_signers.path]\nkeys = [\"{}\"]\n", signer);
    fs::write(root.join("trust.toml"), trust).expect("write trust.toml");

    let verify = run_ocl_cli(&["deps", "verify", &root_s], &empty_env);
    assert_ok(&verify, "deps verify");

    let update = run_ocl_cli(
        &["deps", "update", &root_s, "widgets", "--write-legacy-lock"],
        &empty_env,
    );
    assert_ok(&update, "deps update");
    assert!(root.join("deps.lock.v2").exists(), "missing deps.lock.v2");

    let build = run_ocl_cli(&["pack", "build", &root_s], &empty_env);
    assert_ok(&build, "pack build");
    let artifact = first_oclpkg(&root);
    let artifact_s = artifact.to_string_lossy().to_string();

    let mut tampered = fs::read_to_string(&artifact).expect("read artifact");
    tampered = tampered.replace("signature_ed25519=", "signature_ed25519=broken-");
    fs::write(&artifact, tampered).expect("write tampered artifact");

    let verify_fail = run_ocl_cli(&["pack", "verify", &artifact_s], &empty_env);
    assert!(
        !verify_fail.status.success(),
        "pack verify must fail on tampered signature"
    );

    let sign = run_ocl_cli(&["pack", "sign", &artifact_s], &empty_env);
    assert_ok(&sign, "pack sign");

    let verify_pass = run_ocl_cli(&["pack", "verify", &artifact_s], &empty_env);
    assert_ok(&verify_pass, "pack verify after sign");

    let registry = root.join("registry_v10");
    let registry_s = registry.to_string_lossy().to_string();
    let publish = run_ocl_cli(
        &["pack", "publish", &artifact_s, "--registry", &registry_s],
        &empty_env,
    );
    assert_ok(&publish, "pack publish");
    assert!(
        registry.join("registry.index.v2").exists(),
        "missing registry.index.v2"
    );
}
