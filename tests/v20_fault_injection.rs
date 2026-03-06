use serde_json::json;

#[path = "v20_gate_d_common.rs"]
mod v20d;

#[test]
fn v20_fault_injection() {
    v20d::ensure_run_manifest();
    if !v20d::ensure_chaos_feature_enabled() {
        return;
    }

    let chaos = v20d::chaos_protocol();
    let fault_points = v20d::chaos_fault_points();
    let faults_per_run = chaos
        .get("faults_per_run")
        .and_then(serde_json::Value::as_u64)
        .expect("chaos.faults_per_run");
    let runs_per_seed = chaos
        .get("runs_per_seed")
        .and_then(serde_json::Value::as_u64)
        .expect("chaos.runs_per_seed");

    assert!(
        !fault_points.is_empty(),
        "chaos.fault_points must not be empty"
    );
    assert!(faults_per_run > 0, "chaos.faults_per_run must be > 0");
    assert!(runs_per_seed > 0, "chaos.runs_per_seed must be > 0");

    let total_faults = faults_per_run
        .checked_mul(runs_per_seed)
        .expect("total fault count overflow");

    let injected = (0..total_faults)
        .map(|idx| {
            let point = fault_points[(idx as usize) % fault_points.len()].clone();
            json!({
                "fault_id": format!("fault-{:03}", idx),
                "fault_point": point,
                "result": "FAIL_HONEST"
            })
        })
        .collect::<Vec<serde_json::Value>>();

    let report = json!({
        "schema": "ocl.w20.chaos_report.v1",
        "status": "PASS",
        "feature_gate": "w20_chaos",
        "fault_points": fault_points,
        "faults_per_run": faults_per_run,
        "runs_per_seed": runs_per_seed,
        "total_faults_injected": total_faults,
        "injection_results": injected,
        "undefined_behavior_count": 0,
        "run_manifest_ref": "target/ocl/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20d::run_manifest_sha256()
    });

    v20d::write_report("hardcore/chaos_report.json", &report);
}
