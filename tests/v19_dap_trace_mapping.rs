use std::fs;

use ocp_sdk::{
    dap_breakpoint_mapping_v19, dap_trace_events_from_source_v19, EditorDapBreakpointV19,
};
use serde_json::json;

#[path = "v19_gate_a_common.rs"]
mod v19;

#[test]
fn v19_dap_trace_mapping_contract_is_enforced() {
    v19::ensure_run_manifest();

    let contract_path = v19::contracts_root()
        .join("editor")
        .join("ocp_debug_contract.v1.json");
    let contract = v19::read_json(&contract_path);
    let mapping_expr = contract
        .get("breakpoint_mapping")
        .and_then(serde_json::Value::as_str)
        .expect("breakpoint_mapping");
    assert_eq!(mapping_expr, "(file,span)->breakpoint_id->trace_event_id[]");

    let source_path = v19::repo_root()
        .join("tests")
        .join("fixtures")
        .join("v19")
        .join("dap")
        .join("session.ocp");
    let source = fs::read_to_string(&source_path).expect("read dap fixture");
    let events = dap_trace_events_from_source_v19(&source, 1901).expect("trace events");
    let breakpoints = vec![EditorDapBreakpointV19 {
        breakpoint_id: 100,
        file_id: 1901,
        line: 7,
        column: 1,
    }];
    let mapping = dap_breakpoint_mapping_v19(&contract, &events, &breakpoints).expect("mapping");
    assert_eq!(mapping.len(), 1);

    let report = json!({
        "schema": "ocp.w19.debug.dap_trace_mapping_report.v1",
        "status": "PASS",
        "contract_path": contract_path.to_string_lossy().replace('\\', "/"),
        "breakpoint_mapping": mapping_expr,
        "mapping": mapping,
        "run_manifest_ref": "target/ocp/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report("debug/dap_trace_mapping_report.json", &report);
}
