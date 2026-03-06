#![allow(dead_code)]

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

pub fn user_journey_matrix() -> JsonValue {
    let path = v20c::contracts_root()
        .join("v20")
        .join("user_journey_matrix.v1.json");
    v20c::read_json(&path)
}

pub fn dx_friction_budget() -> JsonValue {
    let path = v20c::contracts_root()
        .join("v20")
        .join("dx_friction_budget.v1.json");
    v20c::read_json(&path)
}

pub fn final_audit_scope() -> JsonValue {
    let path = v20c::contracts_root()
        .join("v20")
        .join("final_audit_scope.v1.json");
    v20c::read_json(&path)
}

pub fn find_journey<'a>(matrix: &'a JsonValue, id: &str) -> &'a JsonValue {
    matrix
        .get("journeys")
        .and_then(serde_json::Value::as_array)
        .expect("journeys array")
        .iter()
        .find(|item| item.get("id").and_then(serde_json::Value::as_str) == Some(id))
        .unwrap_or_else(|| panic!("missing journey id `{id}`"))
}

pub fn journey_steps(journey: &JsonValue) -> Vec<String> {
    journey
        .get("steps")
        .and_then(serde_json::Value::as_array)
        .expect("journey.steps array")
        .iter()
        .map(|item| {
            item.as_str()
                .unwrap_or_else(|| panic!("journey step must be string"))
                .to_string()
        })
        .collect::<Vec<String>>()
}
