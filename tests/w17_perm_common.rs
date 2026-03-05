#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use ocl_sdk::init_project;

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocl_v17_perm_{tag}_{stamp}"))
}

pub fn run_ocl_cli(args: &[&str], envs: &BTreeMap<&str, &str>) -> Output {
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

pub fn assert_ok(output: &Output, step: &str) {
    assert!(
        output.status.success(),
        "{step} failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

pub fn init_demo_project(root: &Path) {
    init_project(root).expect("init project");
}

pub fn write_manifest_with_fs_rule(root: &Path, lane: &str, read_rule: &str) {
    let manifest = format!(
        concat!(
            "[package]\n",
            "name = \"perm_v17_demo\"\n",
            "version = \"0.1.0\"\n\n",
            "[project]\n",
            "lane = \"{}\"\n\n",
            "[dependencies]\n",
            "std = \"0.1.0\"\n\n",
            "[permissions.package]\n",
            "allow = [\"std.log.info\", \"std.fs.read_text\"]\n",
            "deny = []\n\n",
            "[permissions.std_fs]\n",
            "read = [\"{}\"]\n"
        ),
        lane, read_rule
    );
    fs::write(root.join("Ocl.toml"), manifest).expect("write Ocl.toml");
}
