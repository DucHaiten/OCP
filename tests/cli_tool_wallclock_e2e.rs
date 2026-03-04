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
    std::env::temp_dir().join(format!("ocl_cli_tool_wallclock_e2e_{tag}_{stamp}"))
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

fn list_dirs(path: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let read = fs::read_dir(path).expect("read dir");
    for entry in read {
        let p = entry.expect("entry").path();
        if p.is_dir() {
            out.push(p);
        }
    }
    out.sort();
    out
}

#[test]
fn cli_tool_wallclock_template_run_record_replay_pass() {
    let root = temp_project_dir("record_replay");
    let root_str = root.to_string_lossy().to_string();

    let init = run_ocl_cli(&["init", &root_str, "--template", "tool-wallclock"], None);
    assert!(
        init.status.success(),
        "init failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&init.stdout),
        String::from_utf8_lossy(&init.stderr)
    );
    assert!(root.join("Ocl.toml").exists(), "missing Ocl.toml");
    assert!(
        root.join("src").join("main.ocl").exists(),
        "missing src/main.ocl"
    );
    assert!(root.join("README.md").exists(), "missing README.md");

    let run_without = run_ocl_cli(&["run", &root_str], None);
    assert!(
        !run_without.status.success(),
        "quarantine lane must fail without OCL_QUARANTINE=1"
    );

    let run_with = run_ocl_cli(&["run", &root_str], Some("1"));
    assert!(
        run_with.status.success(),
        "run with env failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&run_with.stdout),
        String::from_utf8_lossy(&run_with.stderr)
    );
    assert!(
        root.join("out").join("wallclock.txt").exists(),
        "template should materialize out/wallclock.txt"
    );

    let artifacts_root = root.join(".ocl_artifacts");
    let run_dirs = list_dirs(&artifacts_root);
    assert!(!run_dirs.is_empty(), "missing run artifact dir");
    let run_dir = run_dirs.last().expect("latest run dir");

    let cassette = fs::read_to_string(run_dir.join("cassette").join("cassette.jsonl"))
        .expect("read cassette.jsonl");
    assert!(
        cassette.contains("\"cap\":\"std.time.wallclock.now\""),
        "cassette must include wallclock entry"
    );
    assert!(
        cassette.contains("\"call_id\":"),
        "cassette wallclock entry must include call_id"
    );

    let replay_without = run_ocl_cli(&["replay", &run_dir.to_string_lossy()], None);
    assert!(
        !replay_without.status.success(),
        "replay must fail without OCL_QUARANTINE=1 for quarantine lane"
    );

    let replay_with = run_ocl_cli(&["replay", &run_dir.to_string_lossy()], Some("1"));
    assert!(
        replay_with.status.success(),
        "replay with env failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&replay_with.stdout),
        String::from_utf8_lossy(&replay_with.stderr)
    );
}
