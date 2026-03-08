use serde_json::json;
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
    std::env::temp_dir().join(format!("ocp_v16_cassette_replay_{tag}_{stamp}"))
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

fn set_project_lane(root: &Path, lane: &str) {
    let manifest = root.join("Ocp.toml");
    let raw = fs::read_to_string(&manifest).expect("read Ocp.toml");
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
    fs::write(&manifest, patched).expect("write Ocp.toml");
}

fn latest_artifact_dir(root: &Path) -> PathBuf {
    let artifacts_root = root.join(".ocp_artifacts");
    let read = fs::read_dir(&artifacts_root).expect("read .ocp_artifacts");
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

fn parse_replay_signature(replay_toml: &str) -> Option<String> {
    let mut in_replay = false;
    for line in replay_toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_replay = trimmed == "[replay]";
            continue;
        }
        if !in_replay || trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("signature") {
            if let Some((_, rhs)) = value.split_once('=') {
                let parsed = rhs.trim().trim_matches('"').to_string();
                if !parsed.is_empty() {
                    return Some(parsed);
                }
            }
        }
    }
    None
}

#[test]
fn quarantine_record_replay_offline_signature_is_stable() {
    let root = temp_project_dir("wallclock");
    let root_s = root.to_string_lossy().to_string();

    let init = run_ocp_cli(&["init", &root_s, "--template", "tool-cli"], None);
    assert!(
        init.status.success(),
        "init failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&init.stdout),
        String::from_utf8_lossy(&init.stderr)
    );

    set_project_lane(&root, "quarantine");
    let source = r#"observe("std.time.wallclock.now", "tier2", ctx("scope=v16"), budget(5)) -> now_res;
match now_res {
  OK => { let ready = true; }
  DEGRADED => { let ready = true; }
  INSUFFICIENT => { let ready = true; }
  DEFERRED => { let ready = true; }
}
condition(true);
"#;
    fs::write(root.join("src").join("main.ocp"), source).expect("write source");

    let record = run_ocp_cli(&["run", &root_s], Some("1"));
    assert!(
        record.status.success(),
        "record run failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&record.stdout),
        String::from_utf8_lossy(&record.stderr)
    );

    let run_dir = latest_artifact_dir(&root);
    let signature_record = fs::read_to_string(run_dir.join("signature.txt"))
        .expect("read signature.txt")
        .trim()
        .to_string();
    let replay_toml = fs::read_to_string(run_dir.join("replay.toml")).expect("read replay.toml");
    let replay_signature =
        parse_replay_signature(&replay_toml).expect("replay.toml must contain replay signature");
    assert_eq!(
        signature_record, replay_signature,
        "record signature and replay signature in replay.toml must match"
    );
    assert!(
        run_dir.join("cassette").join("cassette.jsonl").exists(),
        "missing cassette.jsonl in artifact"
    );
    assert!(
        run_dir
            .join("cassette")
            .join("cassette_index.json")
            .exists(),
        "missing cassette_index.json in artifact"
    );
    assert!(
        run_dir.join("cassette").join("cassette_meta.toml").exists(),
        "missing cassette_meta.toml in artifact"
    );

    let replay = run_ocp_cli(&["replay", &run_dir.to_string_lossy()], Some("1"));
    assert!(
        replay.status.success(),
        "replay failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&replay.stdout),
        String::from_utf8_lossy(&replay.stderr)
    );

    let replay_again = run_ocp_cli(&["replay", &run_dir.to_string_lossy()], Some("1"));
    assert!(
        replay_again.status.success(),
        "second replay failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&replay_again.stdout),
        String::from_utf8_lossy(&replay_again.stderr)
    );

    let signature_after_replay = fs::read_to_string(run_dir.join("signature.txt"))
        .expect("read signature.txt after replay")
        .trim()
        .to_string();
    assert_eq!(
        signature_record, signature_after_replay,
        "replay must preserve signature invariance for recorded artifact"
    );

    let out_dir = PathBuf::from("target")
        .join("ocp")
        .join("w16")
        .join("determinism");
    fs::create_dir_all(&out_dir).expect("create w16 determinism output dir");

    let cassette_report = json!({
        "schema": "ocp.w16.determinism.cassette_offline_invariance.v1",
        "run_manifest_ref": "target/ocp/w16/meta/run_manifest.json",
        "lane": "quarantine",
        "record_signature": signature_record,
        "replay_signature": replay_signature,
        "replay_status_ok": replay.status.success(),
        "replay_again_status_ok": replay_again.status.success(),
        "cassette_files_present": true
    });
    fs::write(
        out_dir.join("cassette_offline_invariance_report.json"),
        serde_json::to_string_pretty(&cassette_report)
            .expect("serialize cassette offline invariance report"),
    )
    .expect("write cassette_offline_invariance_report.json");

    let replay_report = json!({
        "schema": "ocp.w16.determinism.replay_invariant.v1",
        "run_manifest_ref": "target/ocp/w16/meta/run_manifest.json",
        "invariants": {
            "quarantine_record_replay_signature_equal": true,
            "quarantine_replay_repeatable": true
        },
        "artifact_dir": run_dir.to_string_lossy(),
    });
    fs::write(
        out_dir.join("replay_invariant_report.json"),
        serde_json::to_string_pretty(&replay_report).expect("serialize replay invariant report"),
    )
    .expect("write replay_invariant_report.json");
}
