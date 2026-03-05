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
    std::env::temp_dir().join(format!("ocl_v16_perm_negative_{tag}_{stamp}"))
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
    fs::write(
        root.join("Ocl.toml"),
        concat!(
            "[package]\n",
            "name = \"perm_negative\"\n",
            "version = \"0.1.0\"\n\n",
            "[project]\n",
            "lane = \"locked_v071\"\n\n",
            "[dependencies]\n",
            "std = \"0.1.0\"\n\n",
            "[permissions.package]\n",
            "allow = [\"std.log.info\"]\n",
            "deny = []\n"
        ),
    )
    .expect("write manifest v1");
}

fn write_manifest_v2(root: &Path) {
    fs::write(
        root.join("Ocl.toml"),
        concat!(
            "[package]\n",
            "name = \"perm_negative\"\n",
            "version = \"0.1.0\"\n\n",
            "[project]\n",
            "lane = \"locked_v071\"\n\n",
            "[dependencies]\n",
            "std = \"0.1.0\"\n\n",
            "[permissions.package]\n",
            "allow = [\"std.log.info\", \"std.fs.read_text\"]\n",
            "deny = []\n\n",
            "[permissions.std_fs]\n",
            "read = [\"./data/**\"]\n"
        ),
    )
    .expect("write manifest v2");
}

#[test]
fn perm_bypass_negative_unapproved_diff_must_fail() {
    let root = temp_project_dir("unapproved_diff");
    init_project(&root).expect("init project");
    write_manifest_v1(&root);

    let root_s = root.to_string_lossy().to_string();
    let old_dir = root.join("old");
    let new_dir = root.join("new");
    fs::create_dir_all(&old_dir).expect("create old dir");
    fs::create_dir_all(&new_dir).expect("create new dir");
    let old_dir_s = old_dir.to_string_lossy().to_string();
    let new_dir_s = new_dir.to_string_lossy().to_string();
    let envs = BTreeMap::new();

    let snap_old = run_ocl_cli(
        &["perm", "snapshot", &root_s, "--out-dir", &old_dir_s],
        &envs,
    );
    assert_ok(&snap_old, "perm snapshot old");

    write_manifest_v2(&root);
    let snap_new = run_ocl_cli(
        &["perm", "snapshot", &root_s, "--out-dir", &new_dir_s],
        &envs,
    );
    assert_ok(&snap_new, "perm snapshot new");

    let old_snapshot = old_dir.join("permissions.snapshot.json");
    let new_snapshot = new_dir.join("permissions.snapshot.json");
    let diff_report = root.join("permission_diff_report.json");
    let approval_file = root.join("permissions.approval.toml");

    let old_snapshot_s = old_snapshot.to_string_lossy().to_string();
    let new_snapshot_s = new_snapshot.to_string_lossy().to_string();
    let diff_report_s = diff_report.to_string_lossy().to_string();
    let approval_file_s = approval_file.to_string_lossy().to_string();

    let diff = run_ocl_cli(
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
        &envs,
    );
    assert!(
        !diff.status.success(),
        "perm diff must fail when approval is missing"
    );
    let stderr = String::from_utf8_lossy(&diff.stderr);
    assert!(
        stderr.contains("RC-PERMISSION-UNAPPROVED"),
        "unexpected stderr: {stderr}"
    );
}
