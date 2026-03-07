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

pub fn write_json_pretty(path: &Path, value: &JsonValue) {
    v100a::write_json_pretty(path, value)
}

pub fn write_report(rel_path: &str, report: &JsonValue) {
    if !rel_path.starts_with("docs/") {
        panic!("Gate 1.0-D reports must live under docs/: {rel_path}");
    }
    v100a::write_report(rel_path, report)
}

pub fn docs_contract_required_sections() -> JsonValue {
    read_json(
        &repo_root()
            .join("contracts")
            .join("docs")
            .join("v1.0")
            .join("docs_required_sections.v1.json"),
    )
}

pub fn docs_contract_feature_matrix() -> JsonValue {
    read_json(
        &repo_root()
            .join("contracts")
            .join("docs")
            .join("v1.0")
            .join("docs_feature_matrix.v1.json"),
    )
}

pub fn docs_contract_language_parity() -> JsonValue {
    read_json(
        &repo_root()
            .join("contracts")
            .join("docs")
            .join("v1.0")
            .join("docs_language_parity_policy.v1.json"),
    )
}

pub fn docs_contract_link_redirect_policy() -> JsonValue {
    read_json(
        &repo_root()
            .join("contracts")
            .join("docs")
            .join("v1.0")
            .join("docs_link_redirect_policy.v1.json"),
    )
}

pub fn docs_contract_cli_surface() -> JsonValue {
    read_json(
        &repo_root()
            .join("contracts")
            .join("docs")
            .join("v1.0")
            .join("docs_cli_surface.v1.json"),
    )
}

pub fn read_text(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| panic!("read {}", path.display()))
}

pub fn list_markdown_files(base: &Path) -> Vec<PathBuf> {
    let mut out = Vec::<PathBuf>::new();
    if !base.exists() {
        return out;
    }
    let mut stack = vec![base.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = fs::read_dir(&dir).unwrap_or_else(|_| panic!("read_dir {}", dir.display()));
        for entry in entries {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path
                .extension()
                .and_then(|v| v.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("md"))
                .unwrap_or(false)
            {
                out.push(path);
            }
        }
    }
    out.sort_by(|lhs, rhs| {
        lhs.to_string_lossy()
            .as_bytes()
            .cmp(rhs.to_string_lossy().as_bytes())
    });
    out
}

pub fn markdown_links(content: &str) -> Vec<String> {
    let bytes = content.as_bytes();
    let mut links = Vec::<String>::new();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] != b'[' {
            i += 1;
            continue;
        }
        let mut j = i + 1;
        while j < bytes.len() && bytes[j] != b']' {
            j += 1;
        }
        if j + 1 >= bytes.len() || bytes[j + 1] != b'(' {
            i += 1;
            continue;
        }
        let mut k = j + 2;
        while k < bytes.len() && bytes[k] != b')' {
            k += 1;
        }
        if k <= bytes.len() {
            if let Ok(raw) = std::str::from_utf8(&bytes[(j + 2)..k]) {
                let target = raw.trim().to_string();
                if !target.is_empty() {
                    links.push(target);
                }
            }
        }
        i = k.saturating_add(1);
    }
    links
}

pub fn markdown_heading_lines(content: &str) -> Vec<String> {
    content
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with('#'))
        .map(|line| line.to_string())
        .collect::<Vec<String>>()
}

pub fn heading_ids(content: &str) -> Vec<String> {
    let mut ids = Vec::<String>::new();
    for line in content.lines() {
        let line = line.trim();
        if !line.starts_with("## [") {
            continue;
        }
        if let Some(end) = line.find(']') {
            let id = line[4..end].trim().to_string();
            if !id.is_empty() {
                ids.push(id);
            }
        }
    }
    ids
}
