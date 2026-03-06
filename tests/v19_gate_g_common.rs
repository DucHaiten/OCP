#![allow(dead_code)]

use std::path::PathBuf;

use serde_json::Value as JsonValue;

#[path = "v19_gate_a_common.rs"]
mod v19;

pub fn repo_root() -> PathBuf {
    v19::repo_root()
}

pub fn ensure_run_manifest() -> JsonValue {
    v19::ensure_run_manifest()
}

pub fn run_manifest_sha256() -> String {
    v19::run_manifest_sha256()
}

pub fn read_json(rel_path: &str) -> JsonValue {
    let path = repo_root().join(rel_path);
    v19::read_json(&path)
}

pub fn write_report(rel_path: &str, report: &JsonValue) {
    v19::write_report(rel_path, report)
}
