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
    std::env::temp_dir().join(format!("ocl_trace_index_v11_{tag}_{stamp}"))
}

fn run_ocl_cli(args: &[&str]) -> Output {
    let cargo_bin = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    Command::new(cargo_bin)
        .current_dir(repo_root())
        .arg("run")
        .arg("-p")
        .arg("ocl-cli")
        .arg("--quiet")
        .arg("--")
        .args(args)
        .output()
        .expect("run ocl-cli")
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

fn latest_artifact_dir(project_root: &Path) -> PathBuf {
    let artifacts_root = project_root.join(".ocl_artifacts");
    let mut dirs = list_dirs(&artifacts_root);
    assert!(!dirs.is_empty(), "missing artifact dir");
    dirs.pop().expect("latest artifact dir")
}

#[test]
fn trace_index_is_written_and_trace_view_accepts_artifact_dir() {
    let root = temp_project_dir("index_written");
    let root_s = root.to_string_lossy().to_string();

    let init = run_ocl_cli(&["init", &root_s, "--template", "mini-game"]);
    assert!(
        init.status.success(),
        "init failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&init.stdout),
        String::from_utf8_lossy(&init.stderr)
    );

    let run = run_ocl_cli(&["run", &root_s]);
    assert!(
        run.status.success(),
        "run failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let run_dir = latest_artifact_dir(&root);
    let audit = fs::read_to_string(run_dir.join("audit.jsonl")).expect("read audit");
    assert!(
        audit
            .lines()
            .next()
            .unwrap_or("")
            .contains("\"t\":\"ProgramStart\""),
        "first audit line must be ProgramStart marker"
    );
    assert!(
        audit.contains("\"trace_schema_version\":2"),
        "audit must contain trace_schema_version=2 marker"
    );

    let trace_index =
        fs::read_to_string(run_dir.join("trace_index.json")).expect("read trace_index.json");
    assert!(
        trace_index.contains("\"trace_schema_version\": 2"),
        "trace_index must contain schema version"
    );
    assert!(
        trace_index.contains("\"by_type\""),
        "trace_index must contain by_type section"
    );

    let view = run_ocl_cli(&["trace", "view", &run_dir.to_string_lossy()]);
    assert!(
        view.status.success(),
        "trace view by artifact_dir failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&view.stdout),
        String::from_utf8_lossy(&view.stderr)
    );
}

#[test]
fn trace_view_legacy_pipe_requires_explicit_flag() {
    let root = temp_project_dir("legacy_pipe_flag");
    let root_s = root.to_string_lossy().to_string();
    let legacy_trace = root.join("legacy.trace");
    let legacy_trace_s = legacy_trace.to_string_lossy().to_string();

    let init = run_ocl_cli(&["init", &root_s, "--template", "mini-game"]);
    assert!(
        init.status.success(),
        "init failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&init.stdout),
        String::from_utf8_lossy(&init.stderr)
    );

    let trace_run = run_ocl_cli(&["trace", "run", &root_s, "--out", &legacy_trace_s]);
    assert!(
        trace_run.status.success(),
        "trace run failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&trace_run.stdout),
        String::from_utf8_lossy(&trace_run.stderr)
    );

    let view_without_flag = run_ocl_cli(&["trace", "view", &legacy_trace_s]);
    assert!(
        !view_without_flag.status.success(),
        "trace view without --legacy-pipe must fail for legacy trace format"
    );

    let view_with_flag = run_ocl_cli(&["trace", "view", &legacy_trace_s, "--legacy-pipe"]);
    assert!(
        view_with_flag.status.success(),
        "trace view with --legacy-pipe failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&view_with_flag.stdout),
        String::from_utf8_lossy(&view_with_flag.stderr)
    );
}
