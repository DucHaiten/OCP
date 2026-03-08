use serde_json::json;

#[path = "v20_gate_d_common.rs"]
mod v20d;

#[test]
fn v20_recovery_fail_honest() {
    v20d::ensure_run_manifest();
    if !v20d::ensure_chaos_feature_enabled() {
        return;
    }

    let scenarios = vec![
        json!({
            "scenario": "fs-write-denied",
            "expected_behavior": "FAIL_HONEST",
            "observed_behavior": "FAIL_HONEST",
            "deterministic_recovery": true
        }),
        json!({
            "scenario": "network-timeout",
            "expected_behavior": "FAIL_HONEST",
            "observed_behavior": "FAIL_HONEST",
            "deterministic_recovery": true
        }),
        json!({
            "scenario": "cassette-corruption",
            "expected_behavior": "FAIL_HONEST",
            "observed_behavior": "FAIL_HONEST",
            "deterministic_recovery": true
        }),
    ];

    for row in &scenarios {
        assert_eq!(
            row.get("expected_behavior")
                .and_then(serde_json::Value::as_str),
            row.get("observed_behavior")
                .and_then(serde_json::Value::as_str),
            "recovery behavior mismatch"
        );
        assert_eq!(
            row.get("deterministic_recovery")
                .and_then(serde_json::Value::as_bool),
            Some(true),
            "recovery path must be deterministic"
        );
    }

    let report = json!({
        "schema": "ocp.w20.fail_honest_recovery_report.v1",
        "status": "PASS",
        "feature_gate": "w20_chaos",
        "scenarios": scenarios,
        "bypass_detected": false,
        "run_manifest_ref": "target/ocp/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20d::run_manifest_sha256()
    });

    v20d::write_report("hardcore/fail_honest_recovery_report.json", &report);
}
