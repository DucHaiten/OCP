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
fn budget_diamond_keeps_edge_scope_by_caller_package() {
    let root = common::temp_dir("budget_diamond");
    let artifact = root.join("artifact");
    write_audit(
        &artifact,
        &[
            json!({
                "t":"TraceEvent","i":1,"tick":1,"seed":42,"call_id":1,"span":JsonValue::Null,
                "data":{
                    "seq":1,"run_id":"r1","event":"observe_end","key":"std.fs.read_text",
                    "callsite_package_id":"pkg.A","kind":"insufficient","reason":"RC-BUDGET-EXCEEDED",
                    "origin_id":11,"allowed":JsonValue::Null,"value":JsonValue::Null,"steps":3,
                    "universe_id":"__legacy__","domain_id":"default","payload_hash":"ha"
                }
            }),
            json!({
                "t":"TraceEvent","i":2,"tick":2,"seed":42,"call_id":2,"span":JsonValue::Null,
                "data":{
                    "seq":2,"run_id":"r1","event":"observe_end","key":"std.fs.read_text",
                    "callsite_package_id":"pkg.B","kind":"insufficient","reason":"RC-BUDGET-EXCEEDED",
                    "origin_id":12,"allowed":JsonValue::Null,"value":JsonValue::Null,"steps":3,
                    "universe_id":"__legacy__","domain_id":"default","payload_hash":"hb"
                }
            }),
            json!({
                "t":"TraceEvent","i":3,"tick":3,"seed":42,"call_id":3,"span":JsonValue::Null,
                "data":{
                    "seq":3,"run_id":"r1","event":"observe_end","key":"std.fs.read_text",
                    "callsite_package_id":"pkg.A","kind":"ok","reason":JsonValue::Null,
                    "origin_id":13,"allowed":JsonValue::Null,"value":JsonValue::Null,"steps":2,
                    "universe_id":"__legacy__","domain_id":"default","payload_hash":"hc"
                }
            }),
        ],
    );

    let artifact_s = artifact.to_string_lossy().to_string();
    let analyze = common::run_ocl_cli(&["budget", "analyze", &artifact_s, "--json"]);
    let analyze_out = common::assert_success(&analyze);
    let parsed: JsonValue = serde_json::from_str(&analyze_out).expect("parse analyze json");
    let edges = parsed
        .get("edges")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(
        edges.iter().any(|edge| {
            edge.get("caller").and_then(JsonValue::as_str) == Some("pkg.A")
                && edge.get("key").and_then(JsonValue::as_str) == Some("std.fs.read_text")
        }),
        "missing edge for pkg.A"
    );
    assert!(
        edges.iter().any(|edge| {
            edge.get("caller").and_then(JsonValue::as_str) == Some("pkg.B")
                && edge.get("key").and_then(JsonValue::as_str) == Some("std.fs.read_text")
        }),
        "missing edge for pkg.B"
    );

    let doctor = common::run_ocl_cli(&["budget", "doctor", &artifact_s, "--json"]);
    let doctor_out = common::assert_success(&doctor);
    let doctor_json: JsonValue = serde_json::from_str(&doctor_out).expect("parse doctor json");
    let issues = doctor_json
        .get("issues")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(
        issues.len() >= 2,
        "expected budget doctor to emit at least two edge issues"
    );
}
