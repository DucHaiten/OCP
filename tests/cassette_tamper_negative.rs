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
    std::env::temp_dir().join(format!("ocl_v16_cassette_negative_{tag}_{stamp}"))
}

fn run_ocl_cli(args: &[&str], quarantine_env: Option<&str>) -> Output {
    let cargo_bin = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut cmd = Command::new(cargo_bin);
    cmd.current_dir(repo_root())
        .arg("run")
        .arg("-p")
        .arg("ocl-cli")
        .arg("--quiet")
        .arg("--")
        .args(args)
        .env_remove("OCL_QUARANTINE");
    if let Some(value) = quarantine_env {
        cmd.env("OCL_QUARANTINE", value);
    }
    cmd.output().expect("run ocl-cli")
}

fn configure_manifest_for_signed_quarantine(root: &Path) {
    let manifest_path = root.join("Ocl.toml");
    let raw = fs::read_to_string(&manifest_path).expect("read Ocl.toml");
    let patched = raw.replace("lane = \"locked_v071\"", "lane = \"quarantine\"")
        + concat!(
            "\n[quarantine]\n",
            "require_signed_cassette = true\n",
            "\n[permissions.package]\n",
            "allow = [\"std.time.wallclock.now\"]\n",
            "deny = []\n"
        );
    fs::write(&manifest_path, patched).expect("write Ocl.toml");
}

fn latest_artifact_dir(root: &Path) -> PathBuf {
    let artifacts_root = root.join(".ocl_artifacts");
    let mut dirs = fs::read_dir(&artifacts_root)
        .expect("read .ocl_artifacts")
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.is_dir() {
                Some(path)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    dirs.sort();
    dirs.pop().expect("missing artifact dir")
}

#[test]
fn cassette_tamper_negative_signature_tamper_must_fail_replay() {
    let root = temp_project_dir("sig_tamper");
    let root_s = root.to_string_lossy().to_string();

    let init = run_ocl_cli(&["init", &root_s, "--template", "tool-cli"], None);
    assert!(
        init.status.success(),
        "init failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&init.stdout),
        String::from_utf8_lossy(&init.stderr)
    );

    configure_manifest_for_signed_quarantine(&root);
    let source = r#"observe("std.time.wallclock.now", "tier2", ctx("scope=cassette_negative"), budget(5)) -> r;
condition(true);
"#;
    fs::write(root.join("src").join("main.ocl"), source).expect("write source");

    let run = run_ocl_cli(&["run", &root_s], Some("1"));
    assert!(
        run.status.success(),
        "run failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let artifact = latest_artifact_dir(&root);
    let sig_path = artifact.join("cassette").join("cassette.sig");
    let raw_sig = fs::read_to_string(&sig_path).expect("read cassette.sig");
    let tampered_sig = raw_sig.replace("signature = \"", "signature = \"tampered-");
    fs::write(&sig_path, tampered_sig).expect("write tampered cassette.sig");

    let replay = run_ocl_cli(&["replay", &artifact.to_string_lossy()], Some("1"));
    assert!(
        !replay.status.success(),
        "replay must fail when cassette signature is tampered"
    );
    let stderr = String::from_utf8_lossy(&replay.stderr);
    assert!(
        stderr.contains("V-CASSETTE-SIGNATURE-INVALID"),
        "unexpected stderr: {stderr}"
    );
}
