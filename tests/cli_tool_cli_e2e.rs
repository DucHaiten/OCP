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
    std::env::temp_dir().join(format!("ocp_cli_tool_v072_{tag}_{stamp}"))
}

fn run_ocp_cli(args: &[&str]) -> Output {
    let cargo_bin = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    Command::new(cargo_bin)
        .current_dir(repo_root())
        .arg("run")
        .arg("-p")
        .arg("ocp-cli")
        .arg("--quiet")
        .arg("--")
        .args(args)
        .output()
        .expect("run ocp-cli")
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
fn cli_tool_cli_template_and_golden_flow_pass() {
    let root = temp_project_dir("golden_pass");
    let root_str = root.to_string_lossy().to_string();

    let init = run_ocp_cli(&["init", &root_str, "--template", "tool-cli"]);
    assert!(
        init.status.success(),
        "init failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&init.stdout),
        String::from_utf8_lossy(&init.stderr)
    );

    assert!(root.join("Ocp.toml").exists(), "missing Ocp.toml");
    assert!(
        root.join("src").join("main.ocp").exists(),
        "missing src/main.ocp"
    );
    assert!(root.join("README.md").exists(), "missing README.md");
    assert!(
        root.join("fixtures")
            .join("in")
            .join("sample.json")
            .exists(),
        "missing fixtures/in/sample.json"
    );
    assert!(
        root.join("fixtures")
            .join("expected")
            .join("out.json")
            .exists(),
        "missing fixtures/expected/out.json"
    );

    let test = run_ocp_cli(&[
        "test",
        &root_str,
        "--golden",
        "fixtures/expected",
        "--clean",
    ]);
    assert!(
        test.status.success(),
        "test failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );

    let artifacts_root = root.join(".ocp_artifacts");
    assert!(artifacts_root.exists(), "missing .ocp_artifacts");
    let run_dirs = list_dirs(&artifacts_root);
    assert!(!run_dirs.is_empty(), "missing run artifact dir");
    let run_dir = &run_dirs[0];

    assert!(run_dir.join("audit.jsonl").exists(), "missing audit.jsonl");
    assert!(
        run_dir.join("signature.txt").exists(),
        "missing signature.txt"
    );
    assert!(run_dir.join("replay.toml").exists(), "missing replay.toml");
    assert!(
        run_dir.join("io").join("fixtures_manifest.json").exists(),
        "missing io/fixtures_manifest.json"
    );
    assert!(
        run_dir.join("state").join("kv_start.json").exists(),
        "missing state/kv_start.json"
    );

    let replay = fs::read_to_string(run_dir.join("replay.toml")).expect("read replay.toml");
    assert!(
        replay.contains("io_mode = \"fixtures\""),
        "replay.toml missing io_mode"
    );
    assert!(
        replay.contains("fixtures_manifest_path = \"io/fixtures_manifest.json\""),
        "replay.toml missing fixtures path"
    );
    assert!(
        replay.contains("kv_start_snapshot_path = \"state/kv_start.json\""),
        "replay.toml missing kv snapshot path"
    );
    assert!(
        replay.contains("kv_start_hash = \""),
        "replay.toml missing kv_start_hash"
    );
}

#[test]
fn cli_tool_cli_golden_mismatch_fails() {
    let root = temp_project_dir("golden_fail");
    let root_str = root.to_string_lossy().to_string();

    let init = run_ocp_cli(&["init", &root_str, "--template", "tool-cli"]);
    assert!(
        init.status.success(),
        "init failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&init.stdout),
        String::from_utf8_lossy(&init.stderr)
    );

    fs::write(
        root.join("fixtures").join("expected").join("out.json"),
        "MISMATCH\n",
    )
    .expect("write mismatch golden");

    let test = run_ocp_cli(&[
        "test",
        &root_str,
        "--golden",
        "fixtures/expected",
        "--clean",
    ]);
    assert!(
        !test.status.success(),
        "test should fail when golden mismatches:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );

    let stderr = String::from_utf8_lossy(&test.stderr);
    assert!(
        stderr.contains("V72-GOLDEN-CONTENT-MISMATCH")
            || stderr.contains("V72-GOLDEN-SHAPE-MISMATCH"),
        "unexpected stderr: {stderr}"
    );
}
