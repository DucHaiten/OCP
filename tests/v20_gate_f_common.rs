#![allow(dead_code)]

use std::path::PathBuf;

use serde_json::Value as JsonValue;

#[path = "v20_gate_c_common.rs"]
mod v20c;

pub fn ensure_run_manifest() -> JsonValue {
    v20c::ensure_run_manifest()
}

pub fn run_manifest_sha256() -> String {
    v20c::run_manifest_sha256()
}

pub fn write_report(rel_path: &str, report: &JsonValue) {
    v20c::write_report(rel_path, report)
}

pub fn contracts_root() -> PathBuf {
    v20c::contracts_root()
}

pub fn repo_root() -> PathBuf {
    contracts_root()
        .parent()
        .expect("contracts root must have parent")
        .to_path_buf()
}

pub fn read_json(path: &std::path::Path) -> JsonValue {
    v20c::read_json(path)
}

pub fn redteam_attack_matrix() -> JsonValue {
    let path = contracts_root()
        .join("v20")
        .join("redteam_attack_matrix.v1.json");
    read_json(&path)
}

pub fn cve_snapshot() -> JsonValue {
    let path = contracts_root().join("v20").join("cve_snapshot.v1.json");
    read_json(&path)
}

pub fn threat_model() -> JsonValue {
    let path = contracts_root().join("v20").join("threat_model.v1.json");
    read_json(&path)
}

pub fn string_array_field(value: &JsonValue, field: &str) -> Vec<String> {
    value
        .get(field)
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("missing array field `{field}`"))
        .iter()
        .map(|item| {
            item.as_str()
                .unwrap_or_else(|| panic!("field `{field}` item must be string"))
                .to_string()
        })
        .collect::<Vec<String>>()
}

pub fn string_array_at_path(value: &JsonValue, path: &[&str]) -> Vec<String> {
    let mut cursor = value;
    for key in path {
        cursor = cursor
            .get(*key)
            .unwrap_or_else(|| panic!("missing key `{key}` in path {:?}", path));
    }
    cursor
        .as_array()
        .unwrap_or_else(|| panic!("path {:?} must resolve to array", path))
        .iter()
        .map(|item| {
            item.as_str()
                .unwrap_or_else(|| panic!("path {:?} item must be string", path))
                .to_string()
        })
        .collect::<Vec<String>>()
}

pub fn contains_secret_marker(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    let has_auth_bearer = lower.contains("authorization: bearer ")
        && !lower.contains("authorization: <redacted>")
        && !lower.contains("authorization: <redacted>");
    let has_api_key = lower.contains("api_key=") && !lower.contains("api_key=<redacted>");
    let has_password = lower.contains("password=") && !lower.contains("password=<redacted>");
    let has_secret = lower.contains("secret=") && !lower.contains("secret=<redacted>");
    let has_token = lower.contains("token=") && !lower.contains("token=<redacted>");
    has_auth_bearer || has_api_key || has_password || has_secret || has_token
}

pub fn redact_sensitive(text: &str) -> String {
    let mut out = text.to_string();
    let replacements = [
        ("Authorization: Bearer ", "Authorization: <REDACTED>"),
        ("authorization: bearer ", "authorization: <redacted>"),
        ("api_key=", "api_key=<REDACTED>"),
        ("password=", "password=<REDACTED>"),
        ("secret=", "secret=<REDACTED>"),
        ("token=", "token=<REDACTED>"),
    ];
    for (needle, replacement) in replacements {
        if out.contains(needle) {
            if needle.ends_with('=') {
                let mut rebuilt = String::new();
                for segment in out.split_whitespace() {
                    if segment.starts_with(needle) {
                        if !rebuilt.is_empty() {
                            rebuilt.push(' ');
                        }
                        rebuilt.push_str(replacement);
                    } else {
                        if !rebuilt.is_empty() {
                            rebuilt.push(' ');
                        }
                        rebuilt.push_str(segment);
                    }
                }
                out = rebuilt;
            } else if let Some((prefix, _)) = out.split_once(needle) {
                out = format!("{prefix}{replacement}");
            }
        }
    }
    out
}
