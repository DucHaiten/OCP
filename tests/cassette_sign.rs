use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocp_cli_cassette_sign_{tag}_{stamp}"))
}

fn run_ocp_cli(args: &[&str], quarantine_env: Option<&str>) -> Output {
    let cargo_bin = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut cmd = Command::new(cargo_bin);
    cmd.current_dir(repo_root())
        .arg("run")
        .arg("-p")
        .arg("ocp-cli")
        .arg("--quiet")
        .arg("--")
        .args(args)
        .env_remove("OCP_QUARANTINE");
    if let Some(value) = quarantine_env {
        cmd.env("OCP_QUARANTINE", value);
    }
    cmd.output().expect("run ocp-cli")
}

fn configure_manifest_for_signed_quarantine(root: &Path) {
    let manifest_path = root.join("Ocp.toml");
    let raw = fs::read_to_string(&manifest_path).expect("read Ocp.toml");
    let patched = raw.replace("lane = \"locked_v071\"", "lane = \"quarantine\"")
        + concat!(
            "\n[quarantine]\n",
            "require_signed_cassette = true\n",
            "\n[permissions.package]\n",
            "allow = [\"std.time.wallclock.now\", \"std.proc.exec\"]\n",
            "deny = []\n",
            "\n[permissions.std_proc]\n",
            "enabled = true\n",
            "allow_bins = [\"mock.proc\"]\n",
            "allow_args_glob = [\"*\"]\n",
            "timeout_ms = 5000\n",
            "max_stdout_bytes = 1048576\n",
            "max_stderr_bytes = 1048576\n"
        );
    fs::write(&manifest_path, patched).expect("write Ocp.toml");
}

fn sha256_hex_text(raw: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    let out = hasher.finalize();
    let mut text = String::with_capacity(out.len() * 2);
    for b in out {
        text.push_str(&format!("{b:02x}"));
    }
    text
}

fn field_from_sig(sig_text: &str, key: &str) -> String {
    let needle = format!("{key} = \"");
    let line = sig_text
        .lines()
        .find(|line| line.trim_start().starts_with(&needle))
        .unwrap_or_else(|| panic!("missing field `{key}` in cassette.sig"));
    let value = line
        .split_once('=')
        .map(|(_, v)| v.trim())
        .unwrap_or("\"\"");
    value.trim_matches('"').to_string()
}

fn rewrite_sig_with_public_forge(sig_path: &Path) {
    let raw = fs::read_to_string(sig_path).expect("read cassette.sig");
    let key_id = field_from_sig(&raw, "key_id");
    let lane = field_from_sig(&raw, "lane");
    let mode = field_from_sig(&raw, "mode");
    let cassette_hash = field_from_sig(&raw, "cassette_hash");
    let signer_pub = sha256_hex_text(&format!("cassette-signer-v15|{key_id}"));
    let signature = sha256_hex_text(&format!(
        "cassette-sign-v15|key_id={key_id}|signer_pub={signer_pub}|lane={lane}|mode={mode}|cassette_hash={cassette_hash}"
    ));

    let mut patched = String::new();
    for line in raw.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("signer_pub = ") {
            patched.push_str(&format!("signer_pub = \"{signer_pub}\"\n"));
        } else if trimmed.starts_with("signature = ") {
            patched.push_str(&format!("signature = \"{signature}\"\n"));
        } else {
            patched.push_str(line);
            patched.push('\n');
        }
    }
    fs::write(sig_path, patched).expect("write forged cassette.sig");
}

fn latest_artifact_dir(root: &Path) -> PathBuf {
    let artifacts_root = root.join(".ocp_artifacts");
    let mut dirs = fs::read_dir(&artifacts_root)
        .expect("read .ocp_artifacts")
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

fn prepare_signed_quarantine_project(root: &Path) -> String {
    let root_s = root.to_string_lossy().to_string();
    let init = run_ocp_cli(&["init", &root_s, "--template", "tool-cli"], None);
    assert!(
        init.status.success(),
        "init failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&init.stdout),
        String::from_utf8_lossy(&init.stderr)
    );

    configure_manifest_for_signed_quarantine(root);
    let source = r#"observe("std.time.wallclock.now", "tier2", ctx("scope=cassette_sign"), budget(5)) -> now_res;
match now_res {
  OK => { let ready = true; }
  DEGRADED => { let ready = true; }
  INSUFFICIENT => { let ready = true; }
  DEFERRED => { let ready = true; }
}
condition(true);
"#;
    fs::write(root.join("src").join("main.ocp"), source).expect("write source");
    root_s
}

#[test]
fn cassette_signing_creates_signature_and_replay_requires_it() {
    let root = temp_project_dir("require_sig");
    let root_s = prepare_signed_quarantine_project(&root);

    let run = run_ocp_cli(&["run", &root_s], Some("1"));
    assert!(
        run.status.success(),
        "run failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let artifact = latest_artifact_dir(&root);
    let sig_path = artifact.join("cassette").join("cassette.sig");
    assert!(sig_path.exists(), "missing cassette.sig");

    let replay_toml =
        fs::read_to_string(artifact.join("replay.toml")).expect("read replay.toml for assertions");
    assert!(
        replay_toml.contains("require_signed_cassette = true"),
        "replay.toml must persist require_signed_cassette=true"
    );

    let replay_ok = run_ocp_cli(&["replay", &artifact.to_string_lossy()], Some("1"));
    assert!(
        replay_ok.status.success(),
        "replay should pass with intact cassette signature:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&replay_ok.stdout),
        String::from_utf8_lossy(&replay_ok.stderr)
    );

    fs::remove_file(&sig_path).expect("remove cassette.sig");
    let replay_missing_sig = run_ocp_cli(&["replay", &artifact.to_string_lossy()], Some("1"));
    assert!(
        !replay_missing_sig.status.success(),
        "replay must fail when cassette.sig is missing"
    );
    let stderr = String::from_utf8_lossy(&replay_missing_sig.stderr);
    assert!(
        stderr.contains("V-CASSETTE-SIGNATURE-MISSING"),
        "expected missing signature error, got: {stderr}"
    );
}

#[test]
fn cassette_signing_detects_tampered_signature_payload() {
    let root = temp_project_dir("tamper_sig");
    let root_s = prepare_signed_quarantine_project(&root);

    let run = run_ocp_cli(&["run", &root_s], Some("1"));
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

    let replay = run_ocp_cli(&["replay", &artifact.to_string_lossy()], Some("1"));
    assert!(
        !replay.status.success(),
        "replay must fail when cassette signature is tampered"
    );
    let stderr = String::from_utf8_lossy(&replay.stderr);
    assert!(
        stderr.contains("V-CASSETTE-SIGNATURE-INVALID"),
        "expected signature invalid error, got: {stderr}"
    );
}

#[test]
fn cassette_signing_rejects_public_forged_signature_recompute() {
    let root = temp_project_dir("forge_public");
    let root_s = prepare_signed_quarantine_project(&root);

    let run = run_ocp_cli(&["run", &root_s], Some("1"));
    assert!(
        run.status.success(),
        "run failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let artifact = latest_artifact_dir(&root);
    let sig_path = artifact.join("cassette").join("cassette.sig");
    rewrite_sig_with_public_forge(&sig_path);

    let replay = run_ocp_cli(&["replay", &artifact.to_string_lossy()], Some("1"));
    assert!(
        !replay.status.success(),
        "replay must fail for forged signature recompute from public fields"
    );
    let stderr = String::from_utf8_lossy(&replay.stderr);
    assert!(
        stderr.contains("V-CASSETTE-SIGNATURE-INVALID"),
        "expected signature invalid error, got: {stderr}"
    );
}

#[test]
fn signed_quarantine_fails_when_proc_env_pattern_leaks() {
    let root = temp_project_dir("env_redact");
    let root_s = prepare_signed_quarantine_project(&root);
    let source = r#"observe("std.proc.exec", "tier2", ctx("bin=mock.proc;args=--stdout=API_TOKEN=leak"), budget(5)) -> r;
condition(true);
"#;
    fs::write(root.join("src").join("main.ocp"), source).expect("write source");

    let run = run_ocp_cli(&["run", &root_s], Some("1"));
    assert!(
        !run.status.success(),
        "run must fail when require_signed_cassette=true and env secret pattern is leaked"
    );
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(
        stderr.contains("V-CASSETTE-REDACTION-REQUIRED"),
        "expected redaction required error, got: {stderr}"
    );
}
