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
    std::env::temp_dir().join(format!("ocp_cli_mini_game_v073_{tag}_{stamp}"))
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

fn count_project_files(root: &Path) -> usize {
    fn walk(path: &Path, count: &mut usize) {
        let Ok(read) = fs::read_dir(path) else {
            return;
        };
        for entry in read {
            let Ok(entry) = entry else {
                continue;
            };
            let p = entry.path();
            if p.file_name()
                .and_then(|s| s.to_str())
                .map(|n| n == ".ocp_artifacts")
                .unwrap_or(false)
            {
                continue;
            }
            if p.is_dir() {
                walk(&p, count);
            } else if p.is_file() {
                *count += 1;
            }
        }
    }
    let mut total = 0usize;
    walk(root, &mut total);
    total
}

#[test]
fn cli_mini_game_template_run_and_replay_pass() {
    let root = temp_project_dir("run_replay");
    let root_str = root.to_string_lossy().to_string();

    let init = run_ocp_cli(&["init", &root_str, "--template", "mini-game"]);
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
        count_project_files(&root) <= 7,
        "template file count must be <= 7"
    );

    let run = run_ocp_cli(&["run", &root_str]);
    assert!(
        run.status.success(),
        "run failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let artifacts_root = root.join(".ocp_artifacts");
    assert!(artifacts_root.exists(), "missing .ocp_artifacts");
    let mut run_dirs = list_dirs(&artifacts_root);
    assert!(!run_dirs.is_empty(), "missing run artifact dir");
    let run_dir = run_dirs.pop().expect("latest run dir");
    assert!(run_dir.join("audit.jsonl").exists(), "missing audit.jsonl");
    assert!(
        run_dir.join("signature.txt").exists(),
        "missing signature.txt"
    );
    assert!(run_dir.join("replay.toml").exists(), "missing replay.toml");

    let replay = run_ocp_cli(&["replay", &run_dir.to_string_lossy()]);
    assert!(
        replay.status.success(),
        "replay failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&replay.stdout),
        String::from_utf8_lossy(&replay.stderr)
    );
}
