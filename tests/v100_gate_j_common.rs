#![allow(dead_code)]

use serde_json::Value as JsonValue;

#[path = "v100_gate_a_common.rs"]
mod v100a;

pub fn ensure_run_manifest() -> JsonValue {
    v100a::ensure_run_manifest()
}

pub fn run_manifest_sha256() -> String {
    v100a::run_manifest_sha256()
}

pub fn write_report(rel_path: &str, report: &JsonValue) {
    if !rel_path.starts_with("ecosystem/") {
        panic!("Gate 1.0-J reports must live under ecosystem/: {rel_path}");
    }
    v100a::write_report(rel_path, report)
}
