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
    std::env::temp_dir().join(format!("ocl_checkpoints_v11_{tag}_{stamp}"))
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

fn extract_json_string_field(raw: &str, field: &str) -> Option<String> {
    let marker = format!("\"{field}\": \"");
    let start = raw.find(&marker)?;
    let remain = &raw[start + marker.len()..];
    let end = remain.find('"')?;
    Some(remain[..end].to_string())
}

#[test]
fn replay_writes_checkpoint_bundle_with_rewind_validation() {
    let root = temp_project_dir("bundle");
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

    let artifact = latest_artifact_dir(&root);
    let replay = run_ocl_cli(&["replay", &artifact.to_string_lossy()]);
    assert!(
        replay.status.success(),
        "replay failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&replay.stdout),
        String::from_utf8_lossy(&replay.stderr)
    );

    let checkpoint_path = artifact.join("checkpoints").join("replay.checkpoints.json");
    assert!(
        checkpoint_path.exists(),
        "checkpoint bundle not written at {}",
        checkpoint_path.display()
    );

    let raw = fs::read_to_string(&checkpoint_path).expect("read checkpoints json");
    assert!(
        raw.contains("\"trace_schema_version\": 2"),
        "checkpoint bundle must include trace schema version"
    );
    assert!(
        raw.contains("\"state_digest_schema_version\": \"v1\""),
        "checkpoint bundle must include state digest schema version"
    );
    assert!(
        raw.contains("\"checkpoint_every\": 200"),
        "checkpoint bundle must use default checkpoint stride"
    );
    assert!(
        raw.contains("\"rewind_match\": true"),
        "checkpoint rewind validation must match"
    );
    assert!(
        raw.contains("\"checkpoints\": ["),
        "checkpoint bundle must include checkpoints array"
    );
    assert!(
        raw.contains("\"event_i\":"),
        "checkpoint bundle must include at least one checkpoint entry"
    );
}

#[test]
fn replay_checkpoint_final_digest_is_stable_across_replays() {
    let root = temp_project_dir("stable_digest");
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

    let artifact = latest_artifact_dir(&root);
    let checkpoint_path = artifact.join("checkpoints").join("replay.checkpoints.json");

    let replay_1 = run_ocl_cli(&["replay", &artifact.to_string_lossy()]);
    assert!(
        replay_1.status.success(),
        "first replay failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&replay_1.stdout),
        String::from_utf8_lossy(&replay_1.stderr)
    );
    let first = fs::read_to_string(&checkpoint_path).expect("read first checkpoint json");
    let first_digest =
        extract_json_string_field(&first, "final_state_digest").expect("first final_state_digest");

    let replay_2 = run_ocl_cli(&["replay", &artifact.to_string_lossy()]);
    assert!(
        replay_2.status.success(),
        "second replay failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&replay_2.stdout),
        String::from_utf8_lossy(&replay_2.stderr)
    );
    let second = fs::read_to_string(&checkpoint_path).expect("read second checkpoint json");
    let second_digest = extract_json_string_field(&second, "final_state_digest")
        .expect("second final_state_digest");

    assert_eq!(
        first_digest, second_digest,
        "final_state_digest must be stable across repeated replay runs"
    );
}
