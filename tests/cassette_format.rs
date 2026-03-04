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
    std::env::temp_dir().join(format!("ocl_cli_cassette_format_{tag}_{stamp}"))
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

fn set_project_lane(root: &Path, lane: &str) {
    let manifest = root.join("Ocl.toml");
    let raw = fs::read_to_string(&manifest).expect("read Ocl.toml");
    let mut in_project = false;
    let mut replaced = false;
    let mut patched = String::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_project = trimmed == "[project]";
        }
        if in_project && trimmed.starts_with("lane = ") {
            patched.push_str(&format!("lane = \"{lane}\"\n"));
            replaced = true;
            continue;
        }
        patched.push_str(line);
        patched.push('\n');
    }

    assert!(replaced, "manifest missing [project].lane");
    fs::write(&manifest, patched).expect("write Ocl.toml");
}

fn latest_artifact_dir(root: &Path) -> PathBuf {
    let artifacts_root = root.join(".ocl_artifacts");
    let read = fs::read_dir(&artifacts_root).expect("read .ocl_artifacts");
    let mut dirs = Vec::new();
    for entry in read {
        let path = entry.expect("entry").path();
        if path.is_dir() {
            dirs.push(path);
        }
    }
    dirs.sort();
    dirs.pop().expect("missing run artifact dir")
}

fn parse_replay_value(replay: &str, key: &str) -> Option<String> {
    for raw in replay.lines() {
        let line = raw.trim();
        let prefix = format!("{key} = ");
        if !line.starts_with(&prefix) {
            continue;
        }
        let value = line[prefix.len()..].trim();
        return Some(value.trim_matches('"').to_string());
    }
    None
}

#[test]
fn cassette_bundle_hash_and_replay_fail_honest_on_tamper() {
    let root = temp_project_dir("bundle");
    let root_s = root.to_string_lossy().to_string();

    let init = run_ocl_cli(&["init", &root_s, "--template", "mini-game"], None);
    assert!(
        init.status.success(),
        "init failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&init.stdout),
        String::from_utf8_lossy(&init.stderr)
    );
    set_project_lane(&root, "quarantine");

    let run = run_ocl_cli(&["run", &root_s], Some("1"));
    assert!(
        run.status.success(),
        "run failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let run_dir = latest_artifact_dir(&root);
    let cassette_dir = run_dir.join("cassette");
    let jsonl =
        fs::read_to_string(cassette_dir.join("cassette.jsonl")).expect("read cassette.jsonl");
    let index = fs::read_to_string(cassette_dir.join("cassette_index.json"))
        .expect("read cassette_index.json");
    let meta = fs::read_to_string(cassette_dir.join("cassette_meta.toml"))
        .expect("read cassette_meta.toml");
    let stored_hash = fs::read_to_string(cassette_dir.join("cassette_hash.txt"))
        .expect("read cassette_hash.txt")
        .trim()
        .to_string();
    let replay = fs::read_to_string(run_dir.join("replay.toml")).expect("read replay.toml");

    assert!(
        meta.contains("schema_version = \"v0.8\""),
        "meta missing schema_version"
    );
    assert!(meta.contains("mode = \"record\""), "meta missing mode");
    assert!(
        meta.contains("hasher_version = \"sha256-v1\""),
        "meta missing hasher_version"
    );
    assert!(
        index.contains("\"call_id_to_entry_id\":{}"),
        "index missing empty call map"
    );
    assert!(
        index.contains("\"next_call_id\":0"),
        "index missing next_call_id"
    );

    let lane = parse_replay_value(&replay, "lane").expect("replay lane");
    let mode = parse_replay_value(&replay, "mode").expect("replay mode");
    let replay_hash = parse_replay_value(&replay, "cassette_hash").expect("replay cassette_hash");

    assert!(
        !jsonl.contains('\r'),
        "cassette.jsonl must be LF-only canonical"
    );
    assert!(
        !index.contains('\r'),
        "cassette_index.json must be LF-only canonical"
    );
    assert_eq!(lane, "quarantine", "lane must be quarantine");
    assert_eq!(mode, "record", "mode must be record");
    assert_eq!(
        replay_hash, stored_hash,
        "replay cassette_hash must match cassette_hash.txt"
    );

    let replay_ok = run_ocl_cli(&["replay", &run_dir.to_string_lossy()], Some("1"));
    assert!(
        replay_ok.status.success(),
        "replay must pass before tamper:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&replay_ok.stdout),
        String::from_utf8_lossy(&replay_ok.stderr)
    );

    fs::write(
        cassette_dir.join("cassette_index.json"),
        format!("{index}\n"),
    )
    .expect("tamper index");
    let replay_fail = run_ocl_cli(&["replay", &run_dir.to_string_lossy()], Some("1"));
    assert!(
        !replay_fail.status.success(),
        "tampered cassette must fail replay"
    );
    let replay_stderr = String::from_utf8_lossy(&replay_fail.stderr);
    assert!(
        replay_stderr.contains("V-CASSETTE-HASH-MISMATCH"),
        "stderr should report hash mismatch, got: {replay_stderr}"
    );
}
