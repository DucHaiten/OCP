use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use ocl_sdk::init_project;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocl_v15_perm_review_{tag}_{stamp}"))
}

fn run_ocl_cli(args: &[&str], envs: &BTreeMap<&str, &str>) -> Output {
    let cargo_bin = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut cmd = Command::new(cargo_bin);
    cmd.current_dir(repo_root())
        .arg("run")
        .arg("-p")
        .arg("ocl-cli")
        .arg("--quiet")
        .arg("--")
        .args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("run ocl-cli")
}

fn assert_ok(output: &Output, step: &str) {
    assert!(
        output.status.success(),
        "{step} failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn write_manifest_v1(root: &Path) {
    let manifest = concat!(
        "[package]\n",
        "name = \"perm_review_demo\"\n",
        "version = \"0.1.0\"\n\n",
        "[project]\n",
        "lane = \"locked_v071\"\n\n",
        "[dependencies]\n",
        "std = \"0.1.0\"\n\n",
        "[permissions.package]\n",
        "allow = [\"std.log.info\"]\n",
        "deny = []\n",
    );
    fs::write(root.join("Ocl.toml"), manifest).expect("write manifest v1");
}

fn write_manifest_v2_with_new_permission(root: &Path) {
    let manifest = concat!(
        "[package]\n",
        "name = \"perm_review_demo\"\n",
        "version = \"0.1.0\"\n\n",
        "[project]\n",
        "lane = \"locked_v071\"\n\n",
        "[dependencies]\n",
        "std = \"0.1.0\"\n\n",
        "[permissions.package]\n",
        "allow = [\"std.log.info\", \"std.fs.read_text\"]\n",
        "deny = []\n\n",
        "[permissions.std_fs]\n",
        "read = [\"./data/**\"]\n",
    );
    fs::write(root.join("Ocl.toml"), manifest).expect("write manifest v2");
}

#[test]
fn v15_perm_review_requires_approval_for_new_permission_diff() {
    let root = temp_project_dir("approval_flow");
    init_project(&root).expect("init project");
    write_manifest_v1(&root);

    let root_s = root.to_string_lossy().to_string();
    let old_dir = root.join("perm_old");
    let new_dir = root.join("perm_new");
    fs::create_dir_all(&old_dir).expect("create old dir");
    fs::create_dir_all(&new_dir).expect("create new dir");
    let old_dir_s = old_dir.to_string_lossy().to_string();
    let new_dir_s = new_dir.to_string_lossy().to_string();

    let empty_env = BTreeMap::new();
    let snapshot_old = run_ocl_cli(
        &["perm", "snapshot", &root_s, "--out-dir", &old_dir_s],
        &empty_env,
    );
    assert_ok(&snapshot_old, "perm snapshot old");
    assert!(
        old_dir.join("permissions.snapshot.json").exists(),
        "old snapshot missing"
    );

    write_manifest_v2_with_new_permission(&root);
    let snapshot_new = run_ocl_cli(
        &["perm", "snapshot", &root_s, "--out-dir", &new_dir_s],
        &empty_env,
    );
    assert_ok(&snapshot_new, "perm snapshot new");
    assert!(
        new_dir.join("permissions.snapshot.json").exists(),
        "new snapshot missing"
    );

    let old_snapshot = old_dir.join("permissions.snapshot.json");
    let new_snapshot = new_dir.join("permissions.snapshot.json");
    let diff_report = root.join("permission_diff_report.json");
    let approval_file = root.join("permissions.approval.toml");
    let old_snapshot_s = old_snapshot.to_string_lossy().to_string();
    let new_snapshot_s = new_snapshot.to_string_lossy().to_string();
    let diff_report_s = diff_report.to_string_lossy().to_string();
    let approval_file_s = approval_file.to_string_lossy().to_string();

    let diff_fail = run_ocl_cli(
        &[
            "perm",
            "diff",
            &old_snapshot_s,
            &new_snapshot_s,
            "--out",
            &diff_report_s,
            "--approval",
            &approval_file_s,
        ],
        &empty_env,
    );
    assert!(
        !diff_fail.status.success(),
        "perm diff must fail before approval"
    );
    let stderr_fail = String::from_utf8_lossy(&diff_fail.stderr);
    assert!(
        stderr_fail.contains("RC-PERMISSION-UNAPPROVED"),
        "unexpected stderr: {stderr_fail}"
    );

    let approve = run_ocl_cli(
        &[
            "perm",
            "approve",
            &diff_report_s,
            "--approval",
            &approval_file_s,
            "--by",
            "ci-bot",
            "--date",
            "2026-03-05",
            "--note",
            "approve permission delta for test",
        ],
        &empty_env,
    );
    assert_ok(&approve, "perm approve");
    assert!(approval_file.exists(), "approval file missing");

    let review_alias = run_ocl_cli(
        &[
            "perm",
            "review",
            &old_snapshot_s,
            &new_snapshot_s,
            "--out",
            &diff_report_s,
            "--approval",
            &approval_file_s,
        ],
        &empty_env,
    );
    assert_ok(&review_alias, "perm review alias");
}
