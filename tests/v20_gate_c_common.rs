#![allow(dead_code)]

use std::env;
use std::fs;
use std::path::PathBuf;

use serde_json::{json, Value as JsonValue};
use sha2::{Digest, Sha256};

#[path = "v20_gate_a_common.rs"]
mod v20a;

pub fn ensure_run_manifest() -> JsonValue {
    v20a::ensure_run_manifest()
}

pub fn run_manifest_sha256() -> String {
    v20a::run_manifest_sha256()
}

pub fn contracts_root() -> PathBuf {
    v20a::contracts_root()
}

pub fn read_json(path: &std::path::Path) -> JsonValue {
    v20a::read_json(path)
}

pub fn write_report(rel_path: &str, report: &JsonValue) {
    v20a::write_report(rel_path, report)
}

pub fn hardcore_protocol() -> JsonValue {
    let path = contracts_root()
        .join("v20")
        .join("hardcore_test_protocol.v1.json");
    read_json(&path)
}

pub fn fuzz_seed_suite() -> JsonValue {
    let path = contracts_root().join("v20").join("fuzz_seed_suite.v1.json");
    read_json(&path)
}

pub fn tooling_versions() -> JsonValue {
    let path = contracts_root()
        .join("v20")
        .join("tooling_versions.v1.json");
    read_json(&path)
}

pub fn sha256_hex_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

pub fn args_digest(args: &[&str]) -> String {
    let joined = args.join("\u{1f}");
    sha256_hex_bytes(joined.as_bytes())
}

pub fn resolve_tool_binary(tool: &str) -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;
    let path_exts = if cfg!(windows) {
        env::var_os("PATHEXT")
            .unwrap_or_else(|| ".EXE;.CMD;.BAT".into())
            .to_string_lossy()
            .split(';')
            .map(|ext| ext.to_ascii_lowercase())
            .collect::<Vec<String>>()
    } else {
        Vec::<String>::new()
    };

    for dir in env::split_paths(&path_var) {
        if cfg!(windows) {
            for ext in &path_exts {
                let candidate = dir.join(format!("{}{}", tool, ext));
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        } else {
            let candidate = dir.join(tool);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

pub fn tool_invocation_record(tool: &str, args: &[&str]) -> JsonValue {
    let args_digest = args_digest(args);
    if let Some(path) = resolve_tool_binary(tool) {
        let bytes = fs::read(&path).unwrap_or_default();
        let binary_sha = if bytes.is_empty() {
            None
        } else {
            Some(sha256_hex_bytes(&bytes))
        };
        let unavailable_reason = if binary_sha.is_none() {
            Some("IO-READ-EMPTY")
        } else {
            None
        };
        return json!({
            "tool_name": tool,
            "tool_args_digest": args_digest,
            "tool_binary_path": path.to_string_lossy().replace('\\', "/"),
            "tool_binary_sha256": binary_sha,
            "tool_binary_sha256_unavailable_reason_code": unavailable_reason
        });
    }

    json!({
        "tool_name": tool,
        "tool_args_digest": args_digest,
        "tool_binary_path": JsonValue::Null,
        "tool_binary_sha256": JsonValue::Null,
        "tool_binary_sha256_unavailable_reason_code": "TOOL-NOT-FOUND-IN-PATH"
    })
}
