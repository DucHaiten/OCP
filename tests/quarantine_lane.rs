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
    std::env::temp_dir().join(format!("ocp_cli_quarantine_lane_{tag}_{stamp}"))
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

#[test]
fn quarantine_lane_requires_env_and_emits_lane_marker() {
    let root = temp_project_dir("gate");
    let root_s = root.to_string_lossy().to_string();

    let init = run_ocp_cli(&["init", &root_s, "--template", "mini-game"], None);
    assert!(
        init.status.success(),
        "init failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&init.stdout),
        String::from_utf8_lossy(&init.stderr)
    );

    set_project_lane(&root, "quarantine");

    let run_without = run_ocp_cli(&["run", &root_s], None);
    assert!(
        !run_without.status.success(),
        "run without env must fail for quarantine lane"
    );
    let stderr_without = String::from_utf8_lossy(&run_without.stderr);
    assert!(
        stderr_without.contains("OCP_QUARANTINE=1"),
        "stderr should mention required quarantine env, got: {stderr_without}"
    );

    let run_with = run_ocp_cli(&["run", &root_s], Some("1"));
    assert!(
        run_with.status.success(),
        "run with env failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&run_with.stdout),
        String::from_utf8_lossy(&run_with.stderr)
    );

    let run_dir = latest_artifact_dir(&root);
    let replay_toml = fs::read_to_string(run_dir.join("replay.toml")).expect("read replay.toml");
    assert!(
        replay_toml.contains("lane = \"quarantine\""),
        "replay.toml must keep quarantine lane"
    );
    let audit = fs::read_to_string(run_dir.join("audit.jsonl")).expect("read audit.jsonl");
    assert!(
        audit.contains("\"t\":\"Lane\""),
        "audit must contain lane marker event"
    );
    assert!(
        audit.contains("\"lane\":\"quarantine\""),
        "audit lane marker must be quarantine"
    );

    let replay_without = run_ocp_cli(&["replay", &run_dir.to_string_lossy()], None);
    assert!(
        !replay_without.status.success(),
        "replay without env must fail for quarantine lane"
    );

    let replay_with = run_ocp_cli(&["replay", &run_dir.to_string_lossy()], Some("1"));
    assert!(
        replay_with.status.success(),
        "replay with env failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&replay_with.stdout),
        String::from_utf8_lossy(&replay_with.stderr)
    );
}

#[test]
fn locked_lane_works_without_quarantine_env() {
    let root = temp_project_dir("locked");
    let root_s = root.to_string_lossy().to_string();

    let init = run_ocp_cli(&["init", &root_s, "--template", "mini-game"], None);
    assert!(
        init.status.success(),
        "init failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&init.stdout),
        String::from_utf8_lossy(&init.stderr)
    );

    let run = run_ocp_cli(&["run", &root_s], None);
    assert!(
        run.status.success(),
        "locked lane run must not require quarantine env:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let run_dir = latest_artifact_dir(&root);
    let replay = run_ocp_cli(&["replay", &run_dir.to_string_lossy()], None);
    assert!(
        replay.status.success(),
        "locked lane replay must not require quarantine env:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&replay.stdout),
        String::from_utf8_lossy(&replay.stderr)
    );
}
