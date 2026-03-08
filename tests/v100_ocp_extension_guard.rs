use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

fn is_text_file(path: &str) -> bool {
    let ext = Path::new(path)
        .extension()
        .and_then(|v| v.to_str())
        .map(|v| v.to_ascii_lowercase());
    matches!(
        ext.as_deref(),
        Some("rs")
            | Some("md")
            | Some("toml")
            | Some("json")
            | Some("yaml")
            | Some("yml")
            | Some("txt")
            | Some("ts")
            | Some("js")
            | Some("ps1")
            | Some("py")
    )
}

fn in_include_roots(path: &str) -> bool {
    path.starts_with("src/")
        || path.starts_with("projects/ocp/crates/")
        || path.starts_with("projects/ocp/apps/")
        || path.starts_with("projects/ocp/conformance/")
        || path.starts_with("tests/")
        || path.starts_with("editor/vscode/ocp/")
        || path.starts_with("contracts/editor/")
        || path.starts_with("tools/")
        || path == "README.md"
        || path == "docs/PROJECT-BOUNDARY.md"
        || path == "docs/OCP-DEBUG-REPLAY-WORKFLOW-v0.11.md"
        || path.starts_with("docs/en/")
        || path.starts_with("docs/vi/")
}

fn is_excluded(path: &str) -> bool {
    path.starts_with("docs/plans/history/")
        || path.contains("/.ocpbundle/")
        || path.contains("/.ocp_artifacts/")
        || path.starts_with("target/")
        || path.contains("/target/")
}

fn legacy_markers() -> (String, String, String, String, String) {
    let lower = String::from_utf8(vec![111, 99, 108]).expect("utf8 lower marker");
    let upper = String::from_utf8(vec![79, 67, 76]).expect("utf8 upper marker");
    let title = String::from_utf8(vec![79, 99, 108]).expect("utf8 title marker");
    let ext = String::from_utf8(vec![46, 111, 99, 108]).expect("utf8 extension marker");
    let phrase = String::from_utf8(vec![
        79, 98, 115, 101, 114, 118, 97, 116, 105, 111, 110, 32, 67, 111, 108, 108, 97, 112, 115,
        101, 32, 76, 97, 110, 103, 117, 97, 103, 101,
    ])
    .expect("utf8 phrase marker");
    (lower, upper, title, ext, phrase)
}

fn has_legacy_marker(text: &str) -> bool {
    let (lower, upper, title, ext, phrase) = legacy_markers();
    text.contains(&upper)
        || text.contains(&title)
        || text.contains(&lower)
        || text.contains(&ext)
        || text.contains(&phrase)
}

#[test]
fn v100_ocp_extension_guard_zero_legacy_tokens() {
    let tracked = run_git(&["ls-files"]);
    assert!(
        tracked.status.success(),
        "git ls-files failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&tracked.stdout),
        String::from_utf8_lossy(&tracked.stderr)
    );
    let tracked_files = parse_lines(&tracked.stdout);

    let forbidden_tracked_legacy_paths = tracked_files
        .iter()
        .filter(|path| {
            let lower = path.to_ascii_lowercase();
            let (marker, _, _, _, _) = legacy_markers();
            lower.contains(&marker)
        })
        .cloned()
        .collect::<Vec<_>>();

    let mut forbidden_legacy_content_paths = Vec::new();
    for path in &tracked_files {
        if !in_include_roots(path) || is_excluded(path) || !is_text_file(path) {
            continue;
        }
        let abs = repo_root().join(path);
        let Ok(content) = fs::read_to_string(&abs) else {
            continue;
        };
        if has_legacy_marker(&content) {
            forbidden_legacy_content_paths.push(path.clone());
        }
    }

    let untracked = run_git(&["ls-files", "--others", "--exclude-standard"]);
    assert!(
        untracked.status.success(),
        "git ls-files --others --exclude-standard failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&untracked.stdout),
        String::from_utf8_lossy(&untracked.stderr)
    );
    let untracked_files = parse_lines(&untracked.stdout);
    let forbidden_untracked_legacy_paths = untracked_files
        .iter()
        .filter(|path| {
            in_include_roots(path) && !is_excluded(path) && {
                let lower = path.to_ascii_lowercase();
                let (marker, _, _, _, _) = legacy_markers();
                lower.contains(&marker)
            }
        })
        .cloned()
        .collect::<Vec<_>>();

    let mut forbidden_untracked_legacy_content_paths = Vec::new();
    for path in &untracked_files {
        if !in_include_roots(path) || is_excluded(path) || !is_text_file(path) {
            continue;
        }
        let abs = repo_root().join(path);
        let Ok(content) = fs::read_to_string(&abs) else {
            continue;
        };
        if has_legacy_marker(&content) {
            forbidden_untracked_legacy_content_paths.push(path.clone());
        }
    }

    assert!(
        forbidden_tracked_legacy_paths.is_empty(),
        "tracked paths still contain legacy marker: {:?}",
        forbidden_tracked_legacy_paths
    );
    assert!(
        forbidden_legacy_content_paths.is_empty(),
        "tracked text files still contain legacy marker: {:?}",
        forbidden_legacy_content_paths
    );
    assert!(
        forbidden_untracked_legacy_paths.is_empty(),
        "untracked paths still contain legacy marker: {:?}",
        forbidden_untracked_legacy_paths
    );
    assert!(
        forbidden_untracked_legacy_content_paths.is_empty(),
        "untracked text files still contain legacy marker: {:?}",
        forbidden_untracked_legacy_content_paths
    );
}
