#![allow(dead_code)]

use serde_json::{json, Value as JsonValue};

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

pub fn sha256_hex_bytes(bytes: &[u8]) -> String {
    v20c::sha256_hex_bytes(bytes)
}

pub fn chaos_protocol() -> JsonValue {
    let protocol = v20c::hardcore_protocol();
    protocol
        .get("chaos")
        .cloned()
        .expect("hardcore.chaos object")
}

pub fn chaos_fault_points() -> Vec<String> {
    chaos_protocol()
        .get("fault_points")
        .and_then(serde_json::Value::as_array)
        .expect("chaos.fault_points")
        .iter()
        .map(|item| item.as_str().expect("fault point string").to_string())
        .collect::<Vec<String>>()
}

pub fn ensure_chaos_feature_enabled() -> bool {
    cfg!(feature = "w20_chaos")
}

pub fn deterministic_schedule() -> Vec<JsonValue> {
    let points = chaos_fault_points();
    let chaos = chaos_protocol();
    let faults_per_run = chaos
        .get("faults_per_run")
        .and_then(serde_json::Value::as_u64)
        .expect("chaos.faults_per_run");
    let runs_per_seed = chaos
        .get("runs_per_seed")
        .and_then(serde_json::Value::as_u64)
        .expect("chaos.runs_per_seed");
    let expected = faults_per_run
        .checked_mul(runs_per_seed)
        .expect("chaos schedule overflow");

    (0..expected)
        .map(|idx| {
            let point = points[(idx as usize) % points.len()].clone();
            json!({
                "slot": idx,
                "fault_point": point,
                "fault_kind": "injected_failure",
                "seed_id": format!("seed-{}", idx % runs_per_seed)
            })
        })
        .collect::<Vec<JsonValue>>()
}
