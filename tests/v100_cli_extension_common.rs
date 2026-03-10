#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

#[path = "w17_gate_b_cli_common.rs"]
mod common;

pub fn init_project_with_template(tag: &str, template: &str) -> PathBuf {
    let root = common::temp_dir(tag);
    let root_s = root.to_string_lossy().to_string();
    let init = common::run_ocp_cli(&["init", &root_s, "--template", template]);
    common::assert_success(&init);
    root
}

pub fn set_project_entry(root: &Path, entry: &str) {
    let manifest_path = root.join("Ocp.toml");
    let mut manifest = fs::read_to_string(&manifest_path).expect("read Ocp.toml");
    manifest.push_str("\n[project]\nentry = \"");
    manifest.push_str(entry);
    manifest.push_str("\"\n");
    fs::write(&manifest_path, manifest).expect("write Ocp.toml");
}

pub fn setup_legacy_oc_project(tag: &str) -> PathBuf {
    let root = init_project_with_template(tag, "mini-game");
    fs::rename(
        root.join("src").join("main.ocp"),
        root.join("src").join("main.oc"),
    )
    .expect("rename src/main.ocp -> src/main.oc");
    set_project_entry(&root, "src/main.oc");
    root
}
