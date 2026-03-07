#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value as JsonValue;

#[path = "v100_gate_a_common.rs"]
mod v100a;

pub fn repo_root() -> PathBuf {
    v100a::repo_root()
}

pub fn ensure_run_manifest() -> JsonValue {
    v100a::ensure_run_manifest()
}

pub fn run_manifest_sha256() -> String {
    v100a::run_manifest_sha256()
}

pub fn read_json(path: &Path) -> JsonValue {
    v100a::read_json(path)
}

pub fn write_report(rel_path: &str, report: &JsonValue) {
    if !rel_path.starts_with("legal/") {
        panic!("Gate 1.0-F reports must live under legal/: {rel_path}");
    }
    v100a::write_report(rel_path, report)
}

pub fn read_text(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| panic!("read {}", path.display()))
}

pub fn legal_pack_required() -> JsonValue {
    read_json(
        &repo_root()
            .join("contracts")
            .join("legal")
            .join("v1.0")
            .join("legal_pack_required.v1.json"),
    )
}

pub fn governance_policy() -> JsonValue {
    read_json(
        &repo_root()
            .join("contracts")
            .join("legal")
            .join("v1.0")
            .join("governance_policy.v1.json"),
    )
}

pub fn trademark_policy() -> JsonValue {
    read_json(
        &repo_root()
            .join("contracts")
            .join("legal")
            .join("v1.0")
            .join("trademark_policy.v1.json"),
    )
}

pub fn licensing_model() -> JsonValue {
    read_json(
        &repo_root()
            .join("contracts")
            .join("business")
            .join("v1.0")
            .join("licensing_model.v1.json"),
    )
}

pub fn community_policy() -> JsonValue {
    read_json(
        &repo_root()
            .join("contracts")
            .join("business")
            .join("v1.0")
            .join("community_contribution_policy.v1.json"),
    )
}

pub fn readme_lines_without_comments(path: &Path) -> Vec<String> {
    read_text(path)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.to_string())
        .collect::<Vec<String>>()
}
