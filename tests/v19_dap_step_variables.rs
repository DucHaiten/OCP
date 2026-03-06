use std::fs;

use ocl_sdk::{
    dap_step_sequence_v19, dap_trace_events_from_source_v19, dap_variables_for_event_v19,
};
use serde_json::json;

#[path = "v19_gate_a_common.rs"]
mod v19;

#[test]
fn v19_dap_step_variables_follow_trace_order() {
    v19::ensure_run_manifest();

    let source_path = v19::repo_root()
        .join("tests")
        .join("fixtures")
        .join("v19")
        .join("dap")
        .join("session.ocl");
    let source = fs::read_to_string(&source_path).expect("read dap fixture");
    let events = dap_trace_events_from_source_v19(&source, 1901).expect("trace events");
    assert!(!events.is_empty(), "events must not be empty");

    let steps = dap_step_sequence_v19(&events);
    assert_eq!(
        steps.len(),
        events.len(),
        "step sequence length must match event count"
    );
    for window in steps.windows(2) {
        assert!(window[0] < window[1], "step ids must be strictly ascending");
    }

    let first_with_vars = events
        .iter()
        .find(|event| !event.variables.is_empty())
        .expect("fixture must produce variable snapshot");
    let vars = dap_variables_for_event_v19(&events, first_with_vars.trace_event_id)
        .expect("variables at event");
    assert!(
        !vars.is_empty(),
        "variables should be available at variable-producing step"
    );

    let report = json!({
        "schema": "ocl.w19.debug.dap_smoke_report.v1",
        "status": "PASS",
        "phase": "step_variables",
        "steps": steps,
        "sample_event_id": first_with_vars.trace_event_id,
        "sample_variables": vars,
        "run_manifest_ref": "target/ocl/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report("debug/dap_smoke_report.json", &report);
}
