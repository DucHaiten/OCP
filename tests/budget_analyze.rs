use std::fs;

use serde_json::{json, Value as JsonValue};

#[path = "w17_gate_b_cli_common.rs"]
mod common;

fn write_audit(artifact: &std::path::Path, events: &[JsonValue]) {
    fs::create_dir_all(artifact).expect("create artifact dir");
    let mut lines = Vec::new();
    lines.push(
        serde_json::to_string(&json!({
            "t":"ProgramStart",
            "i":0,
            "tick":0,
            "seed":0,
            "call_id": JsonValue::Null,
            "span": JsonValue::Null,
            "data": {
                "trace_schema_version": 2,
                "lane": "locked_v071"
            }
        }))
        .expect("serialize program start"),
    );
    for event in events {
        lines.push(serde_json::to_string(event).expect("serialize event"));
    }
    fs::write(artifact.join("audit.jsonl"), lines.join("\n")).expect("write audit");
}

#[test]
fn budget_analyze_outputs_machine_readable_budget_path() {
    let root = common::temp_dir("budget_analyze");
    let artifact = root.join("artifact");
    write_audit(
        &artifact,
        &[
            json!({
                "t":"TraceEvent","i":1,"tick":1,"seed":42,"call_id":1,"span":JsonValue::Null,
                "data":{
                    "seq":1,"run_id":"r1","event":"observe_end","key":"std.fs.read_text",
                    "callsite_package_id":"pkg.core","kind":"insufficient","reason":"RC-BUDGET-EXCEEDED",
                    "origin_id":1,"allowed":JsonValue::Null,"value":JsonValue::Null,"steps":3,
                    "universe_id":"__legacy__","domain_id":"default","payload_hash":"h1"
                }
            }),
            json!({
                "t":"TraceEvent","i":2,"tick":2,"seed":42,"call_id":2,"span":JsonValue::Null,
                "data":{
                    "seq":2,"run_id":"r1","event":"observe_end","key":"std.kv.get",
                    "callsite_package_id":"pkg.core","kind":"ok","reason":JsonValue::Null,
                    "origin_id":2,"allowed":JsonValue::Null,"value":JsonValue::Null,"steps":2,
                    "universe_id":"__legacy__","domain_id":"default","payload_hash":"h2"
                }
            }),
        ],
    );

    let artifact_s = artifact.to_string_lossy().to_string();
    let out = common::run_ocl_cli(&["budget", "analyze", &artifact_s, "--json"]);
    let stdout = common::assert_success(&out);
    let parsed: JsonValue = serde_json::from_str(&stdout).expect("parse budget analyze json");

    assert_eq!(
        parsed
            .get("observe_events")
            .and_then(JsonValue::as_u64)
            .unwrap_or(0),
        2
    );
    assert_eq!(
        parsed
            .get("budget_pressure_events")
            .and_then(JsonValue::as_u64)
            .unwrap_or(0),
        1
    );
    let path = parsed
        .get("budget_path")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    assert_eq!(path.len(), 1, "expected exactly one budget pressure path");
    assert_eq!(
        path[0]
            .get("reason")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "RC-BUDGET-EXCEEDED"
    );
}
