use std::fs;
use std::path::PathBuf;
use std::process::Command;

use serde_json::json;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn run_git(args: &[&str]) -> std::process::Output {
    Command::new("git")
        .current_dir(repo_root())
        .args(args)
        .output()
        .expect("run git")
}

fn parse_lines(output: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(output)
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

fn is_allowed_tracked_artifact(path: &str) -> bool {
    matches!(
        path,
        "projects/ocp-ocl/apps/composer-demo/target/ocl/composer/cache.v1"
            | "projects/ocp-ocl/conformance/fixtures/quarantine-wallclock-replay/.ocl_artifacts/conformance.replay/replay.toml"
    )
}

#[test]
fn v16_repo_hygiene_report_has_no_forbidden_tracked_artifacts_or_secrets() {
    let tracked = run_git(&["ls-files"]);
    assert!(
        tracked.status.success(),
        "git ls-files failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&tracked.stdout),
        String::from_utf8_lossy(&tracked.stderr)
    );
    let tracked_files = parse_lines(&tracked.stdout);

    let forbidden_tracked = tracked_files
        .iter()
        .filter(|path| {
            (path.starts_with("target/")
                || path.contains("/target/")
                || path.starts_with(".ocl_artifacts/")
                || path.contains("/.ocl_artifacts/")
                || path.contains("__pycache__"))
                && !is_allowed_tracked_artifact(path)
        })
        .cloned()
        .collect::<Vec<_>>();

    let suspected_secrets = tracked_files
        .iter()
        .filter(|path| {
            let lower = path.to_ascii_lowercase();
            lower.ends_with(".pem")
                || lower.ends_with(".key")
                || lower.contains("id_rsa")
                || lower.contains("private_key")
                || lower.contains("secret")
        })
        .cloned()
        .collect::<Vec<_>>();

    let status = run_git(&["status", "--porcelain"]);
    assert!(
        status.status.success(),
        "git status --porcelain failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&status.stdout),
        String::from_utf8_lossy(&status.stderr)
    );
    let dirty_entries = parse_lines(&status.stdout);

    assert!(
        forbidden_tracked.is_empty(),
        "tracked forbidden artifacts found: {:?}",
        forbidden_tracked
    );
    assert!(
        suspected_secrets.is_empty(),
        "tracked secret-like files found: {:?}",
        suspected_secrets
    );

    let out_dir = repo_root()
        .join("target")
        .join("ocl")
        .join("w16")
        .join("security");
    fs::create_dir_all(&out_dir).expect("create w16 security output dir");
    let report = json!({
        "schema": "ocl.w16.security.repo_hygiene.v1",
        "run_manifest_ref": "target/ocl/w16/meta/run_manifest.json",
        "repo_clean": dirty_entries.is_empty(),
        "dirty_entries_count": dirty_entries.len(),
        "dirty_entries_sample": dirty_entries.iter().take(20).cloned().collect::<Vec<String>>(),
        "forbidden_tracked_artifacts": forbidden_tracked,
        "suspected_secret_files": suspected_secrets,
        "pass": true
    });
    fs::write(
        out_dir.join("repo_hygiene_report.json"),
        serde_json::to_string_pretty(&report).expect("serialize repo_hygiene report"),
    )
    .expect("write repo_hygiene_report.json");
}
