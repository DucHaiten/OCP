use serde_json::json;

#[path = "v20_gate_d_common.rs"]
mod v20d;

#[test]
fn v20_chaos_schedule_contract() {
    v20d::ensure_run_manifest();
    if !v20d::ensure_chaos_feature_enabled() {
        return;
    }

    let schedule_a = v20d::deterministic_schedule();
    let schedule_b = v20d::deterministic_schedule();
    assert_eq!(
        schedule_a, schedule_b,
        "chaos schedule must be deterministic for identical config"
    );

    let chaos = v20d::chaos_protocol();
    let faults_per_run = chaos
        .get("faults_per_run")
        .and_then(serde_json::Value::as_u64)
        .expect("chaos.faults_per_run");
    let runs_per_seed = chaos
        .get("runs_per_seed")
        .and_then(serde_json::Value::as_u64)
        .expect("chaos.runs_per_seed");
    let expected = faults_per_run
        .checked_mul(runs_per_seed)
        .expect("schedule size overflow");
    assert_eq!(
        schedule_a.len() as u64,
        expected,
        "schedule length must match faults_per_run * runs_per_seed"
    );

    let schedule_digest = {
        let payload = serde_json::to_string(&schedule_a).expect("serialize schedule");
        v20d::sha256_hex_bytes(payload.as_bytes())
    };

    let report = json!({
        "schema": "ocp.w20.chaos_schedule_report.v1",
        "status": "PASS",
        "feature_gate": "w20_chaos",
        "faults_per_run": faults_per_run,
        "runs_per_seed": runs_per_seed,
        "schedule_length": schedule_a.len(),
        "schedule_digest": schedule_digest,
        "schedule_preview": schedule_a.iter().take(8).cloned().collect::<Vec<serde_json::Value>>(),
        "run_manifest_ref": "target/ocp/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20d::run_manifest_sha256()
    });

    v20d::write_report("hardcore/chaos_schedule_report.json", &report);
}
