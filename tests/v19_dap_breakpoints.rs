use std::fs;

use ocp_sdk::{
    dap_breakpoint_mapping_v19, dap_trace_events_from_source_v19, EditorDapBreakpointV19,
};
use serde_json::json;

#[path = "v19_gate_a_common.rs"]
mod v19;

#[test]
fn v19_dap_breakpoints_mapping_is_deterministic() {
    v19::ensure_run_manifest();

    let contract_path = v19::contracts_root()
        .join("editor")
        .join("ocp_debug_contract.v1.json");
    let contract = v19::read_json(&contract_path);
    let source_path = v19::repo_root()
        .join("tests")
        .join("fixtures")
        .join("v19")
        .join("dap")
        .join("session.ocp");
    let source = fs::read_to_string(&source_path).expect("read dap fixture");
    let events = dap_trace_events_from_source_v19(&source, 1901).expect("trace events");

    let breakpoints = vec![
        EditorDapBreakpointV19 {
            breakpoint_id: 1,
            file_id: 1901,
            line: 1,
            column: 1,
        },
        EditorDapBreakpointV19 {
            breakpoint_id: 2,
            file_id: 1901,
            line: 7,
            column: 1,
        },
    ];

    let mapping_a =
        dap_breakpoint_mapping_v19(&contract, &events, &breakpoints).expect("mapping A");
    let mapping_b =
        dap_breakpoint_mapping_v19(&contract, &events, &breakpoints).expect("mapping B");
    assert_eq!(
        mapping_a, mapping_b,
        "breakpoint mapping must be deterministic"
    );
    assert!(
        mapping_a
            .iter()
            .any(|entry| !entry.trace_event_ids.is_empty()),
        "at least one breakpoint must map to trace events"
    );

    let report = json!({
        "schema": "ocp.w19.debug.dap_smoke_report.v1",
        "status": "PASS",
        "phase": "breakpoints",
        "mapping": mapping_a,
        "run_manifest_ref": "target/ocp/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report("debug/dap_smoke_report.json", &report);
}
