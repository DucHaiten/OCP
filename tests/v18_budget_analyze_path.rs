use std::collections::BTreeMap;
use std::fs;

use serde_json::{json, Value as JsonValue};

#[path = "v18_gate_c_common.rs"]
mod common;

#[test]
fn v18_budget_analyze_path_outputs_machine_readable_path_and_report() {
    common::ensure_run_manifest();

    let root = common::temp_project_dir("budget_analyze");
    let artifact = root.join("artifact");
    common::write_budget_analyze_fixture(&artifact);

    let artifact_s = artifact.to_string_lossy().to_string();
    let out = common::run_ocl_cli(
        &["budget", "analyze", &artifact_s, "--json"],
        &BTreeMap::new(),
    );
    common::assert_ok(&out, "budget analyze");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let parsed: JsonValue = serde_json::from_str(&stdout).expect("parse budget analyze json");
    assert_eq!(
        parsed
            .get("observe_events")
            .and_then(JsonValue::as_u64)
            .unwrap_or(0),
        2
    );
    let path = parsed
        .get("budget_path")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    assert_eq!(path.len(), 1, "expected one budget pressure path");
    assert_eq!(
        path[0]
            .get("reason")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "RC-BUDGET-EXCEEDED"
    );

    let out_dir = common::w18_ops_dir();
    fs::create_dir_all(&out_dir).expect("create w18 ops dir");
    let report = json!({
        "schema": "ocl.w18.ops.budget_analyze_report.v1",
        "run_manifest_ref": "target/ocl/w18/meta/run_manifest.json",
        "artifact_dir": artifact.to_string_lossy().replace('\\', "/"),
        "observe_events": parsed.get("observe_events").cloned().unwrap_or(JsonValue::from(0)),
        "budget_pressure_events": parsed.get("budget_pressure_events").cloned().unwrap_or(JsonValue::from(0)),
        "budget_path": parsed.get("budget_path").cloned().unwrap_or(JsonValue::Array(Vec::new())),
        "status": "PASS"
    });
    common::write_json_pretty(&out_dir.join("budget_analyze_report.json"), &report);
}
