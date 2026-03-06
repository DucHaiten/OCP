use ocl_sdk::{dap_trace_path_v19, debug_contract_profile_v19};
use serde_json::json;

#[path = "v19_gate_a_common.rs"]
mod v19;

#[test]
fn v19_dap_trace_acquisition_scheme_is_deterministic() {
    v19::ensure_run_manifest();

    let contract_path = v19::contracts_root()
        .join("editor")
        .join("ocl_debug_contract.v1.json");
    let contract = v19::read_json(&contract_path);
    let (_debug_mode, trace_acquisition, _scheme, _step_semantics) =
        debug_contract_profile_v19(&contract).expect("debug contract profile");
    assert_eq!(trace_acquisition, "generate_on_launch");

    let a = dap_trace_path_v19(&contract, "workspace_a", "run_1").expect("path a");
    let b = dap_trace_path_v19(&contract, "workspace_a", "run_1").expect("path b");
    let c = dap_trace_path_v19(&contract, "workspace_a", "run_2").expect("path c");
    assert_eq!(a, b, "same inputs must yield same trace path");
    assert_ne!(a, c, "different run_id must yield different trace path");
    assert!(
        a.starts_with(".ocl_artifacts/editor_dbg/workspace_a/run_1"),
        "trace path must follow contract scheme"
    );

    let report = json!({
        "schema": "ocl.w19.debug.dap_trace_acquisition_report.v1",
        "status": "PASS",
        "contract_path": contract_path.to_string_lossy().replace('\\', "/"),
        "trace_acquisition": trace_acquisition,
        "sample_path": a,
        "run_manifest_ref": "target/ocl/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report("debug/dap_trace_acquisition_report.json", &report);
}
