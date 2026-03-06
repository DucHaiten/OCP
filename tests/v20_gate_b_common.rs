#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map as JsonMap, Value as JsonValue};
use sha2::{Digest, Sha256};

#[path = "v20_gate_a_common.rs"]
mod v20a;

pub fn ensure_run_manifest() -> JsonValue {
    v20a::ensure_run_manifest()
}

pub fn run_manifest_sha256() -> String {
    v20a::run_manifest_sha256()
}

pub fn repo_root() -> PathBuf {
    v20a::repo_root()
}

pub fn contracts_root() -> PathBuf {
    v20a::contracts_root()
}

pub fn w20_target_root() -> PathBuf {
    v20a::w20_target_root()
}

pub fn write_json_pretty(path: &Path, value: &JsonValue) {
    v20a::write_json_pretty(path, value);
}

pub fn read_json(path: &Path) -> JsonValue {
    v20a::read_json(path)
}

pub fn sha256_hex_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

pub fn sha256_hex_file(path: &Path) -> String {
    let bytes = fs::read(path).unwrap_or_else(|_| panic!("read {}", path.display()));
    sha256_hex_bytes(&bytes)
}

pub fn canonicalize_json_value(value: &JsonValue) -> JsonValue {
    match value {
        JsonValue::Object(map) => {
            let mut sorted = JsonMap::new();
            let mut keys = map.keys().cloned().collect::<Vec<String>>();
            keys.sort_by(|lhs, rhs| lhs.as_bytes().cmp(rhs.as_bytes()));
            for key in keys {
                let item = map.get(&key).expect("json key");
                sorted.insert(key, canonicalize_json_value(item));
            }
            JsonValue::Object(sorted)
        }
        JsonValue::Array(items) => JsonValue::Array(
            items
                .iter()
                .map(canonicalize_json_value)
                .collect::<Vec<JsonValue>>(),
        ),
        _ => value.clone(),
    }
}

pub fn canonical_json_string(value: &JsonValue) -> String {
    let canonical = canonicalize_json_value(value);
    serde_json::to_string(&canonical).expect("canonical json string")
}

pub fn required_versions_v20_replay() -> Vec<&'static str> {
    vec![
        "v0.1", "v0.2", "v0.3", "v0.4", "v0.5", "v0.6", "v0.7.2", "v0.7.3", "v0.8", "v0.9",
        "v0.10", "v0.11", "v0.12", "v0.13", "v0.14", "v0.15", "v0.16", "v0.17", "v0.18", "v0.19",
    ]
}

pub fn plan_file_for_version(version_id: &str) -> Option<&'static str> {
    match version_id {
        "v0.1" => Some("OCP-OCL-MVP-PLAN-v0.1.md"),
        "v0.2" => Some("OCP-OCL-MVP-PLAN-v0.2.md"),
        "v0.3" => Some("OCP-OCL-MVP-PLAN-v0.3.md"),
        "v0.4" => Some("OCP-OCL-MVP-PLAN-v0.4.md"),
        "v0.5" => Some("OCP-OCL-MVP-PLAN-v0.5.md"),
        "v0.6" => Some("OCP-OCL-MVP-PLAN-v0.6.md"),
        "v0.7.2" => Some("OCP-OCL-MVP-PLAN-v0.7.2.md"),
        "v0.7.3" => Some("OCP-OCL-MVP-PLAN-v0.7.3.md"),
        "v0.8" => Some("OCP-OCL-MVP-PLAN-v0.8.md"),
        "v0.9" => Some("OCP-OCL-MVP-PLAN-v0.9.md"),
        "v0.10" => Some("OCP-OCL-MVP-PLAN-v0.10.md"),
        "v0.11" => Some("OCP-OCL-MVP-PLAN-v0.11.md"),
        "v0.12" => Some("OCP-OCL-MVP-PLAN-v0.12.md"),
        "v0.13" => Some("OCP-OCL-MVP-PLAN-v0.13.md"),
        "v0.14" => Some("OCP-OCL-MVP-PLAN-v0.14.md"),
        "v0.15" => Some("OCP-OCL-MVP-PLAN-v0.15.md"),
        "v0.16" => Some("OCP-OCL-MVP-PLAN-v0.16.md"),
        "v0.17" => Some("OCP-OCL-MVP-PLAN-v0.17.md"),
        "v0.18" => Some("OCP-OCL-MVP-PLAN-v0.18.md"),
        "v0.19" => Some("OCP-OCL-MVP-PLAN-v0.19.md"),
        _ => None,
    }
}

pub fn current_os_profile() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => "win-x64",
        ("linux", "x86_64") => "linux-x64",
        ("macos", "aarch64") => "macos-arm64",
        _ => "unsupported",
    }
}

pub fn expected_os_profiles() -> Vec<&'static str> {
    vec!["win-x64", "linux-x64", "macos-arm64"]
}

pub fn write_report(rel_path: &str, value: &JsonValue) {
    let out = w20_target_root().join(rel_path);
    write_json_pretty(&out, value);
}
