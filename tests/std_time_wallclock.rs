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
    std::env::temp_dir().join(format!("ocl_cli_std_time_wallclock_{tag}_{stamp}"))
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

#[test]
fn std_time_wallclock_record_and_replay_use_cassette() {
    let root = temp_project_dir("record_replay");
    let root_s = root.to_string_lossy().to_string();

    let init = run_ocl_cli(&["init", &root_s, "--template", "tool-cli"], None);
    assert!(
        init.status.success(),
        "init failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&init.stdout),
        String::from_utf8_lossy(&init.stderr)
    );

    set_project_lane(&root, "quarantine");

    let source = r#"observe("std.time.wallclock.now", "tier2", ctx("scope=tool"), budget(5)) -> now_res;
match now_res {
  OK => { let ready = true; }
  DEGRADED => { let ready = true; }
  INSUFFICIENT => { let ready = true; }
  DEFERRED => { let ready = true; }
}
condition(true);
"#;
    fs::write(root.join("src").join("main.ocl"), source).expect("write source");

    let run_without = run_ocl_cli(&["run", &root_s], None);
    assert!(
        !run_without.status.success(),
        "quarantine wallclock run without env must fail"
    );

    let run_with = run_ocl_cli(&["run", &root_s], Some("1"));
    assert!(
        run_with.status.success(),
        "run with quarantine env failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&run_with.stdout),
        String::from_utf8_lossy(&run_with.stderr)
    );

    let run_dir = latest_artifact_dir(&root);
    let cassette = fs::read_to_string(run_dir.join("cassette").join("cassette.jsonl"))
        .expect("read cassette.jsonl");
    assert!(
        cassette.contains("\"cap\":\"std.time.wallclock.now\""),
        "cassette must include wallclock cap entry"
    );
    assert!(
        cassette.contains("\"call_id\":0"),
        "first wallclock entry must use call_id 0"
    );

    let replay_without = run_ocl_cli(&["replay", &run_dir.to_string_lossy()], None);
    assert!(
        !replay_without.status.success(),
        "replay without quarantine env must fail for lane quarantine"
    );

    let replay_with = run_ocl_cli(&["replay", &run_dir.to_string_lossy()], Some("1"));
    assert!(
        replay_with.status.success(),
        "replay with quarantine env failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&replay_with.stdout),
        String::from_utf8_lossy(&replay_with.stderr)
    );
}
