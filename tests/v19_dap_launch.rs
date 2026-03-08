use std::fs;

use ocp_sdk::dap_launch_summary_v19;
use serde_json::json;

#[path = "v19_gate_a_common.rs"]
mod v19;

#[test]
fn v19_dap_launch_smoke_contract() {
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

    let summary =
        dap_launch_summary_v19(&contract, "ws_demo", "run001", &source, 1901).expect("dap launch");
    assert_eq!(summary.debug_mode, "dap_replay_backed");
    assert_eq!(summary.trace_acquisition, "generate_on_launch");
    assert_eq!(summary.step_semantics, "trace_event_id_ascending");
    assert!(
        summary
            .trace_path
            .starts_with(".ocp_artifacts/editor_dbg/ws_demo/run001"),
        "trace path must follow deterministic scheme"
    );
    assert!(summary.events_count > 0, "launch must produce trace events");

    let report = json!({
        "schema": "ocp.w19.debug.dap_smoke_report.v1",
        "status": "PASS",
        "phase": "launch",
        "summary": summary,
        "contract_path": contract_path.to_string_lossy().replace('\\', "/"),
        "run_manifest_ref": "target/ocp/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report("debug/dap_smoke_report.json", &report);
}
