use ocp_sdk::runtime_resilience_profile_v19;
use serde_json::json;

#[path = "v19_gate_a_common.rs"]
mod v19;

#[test]
fn v19_lsp_runtime_resilience_contract_is_valid() {
    v19::ensure_run_manifest();

    let policy_path = v19::contracts_root()
        .join("editor")
        .join("editor_runtime_resilience.v1.json");
    let policy = v19::read_json(&policy_path);
    let (max_retries, cooldown_ms, degraded_mode) = runtime_resilience_profile_v19(&policy);

    assert!(max_retries >= 1, "max_retries must be >= 1");
    assert!(cooldown_ms >= 100, "cooldown must be >= 100ms");
    assert!(degraded_mode, "degraded_mode must be true");

    let report = json!({
        "schema": "ocp.w19.lsp.runtime_resilience_report.v1",
        "status": "PASS",
        "max_retries": max_retries,
        "cooldown_ms": cooldown_ms,
        "degraded_mode": degraded_mode,
        "policy_path": policy_path.to_string_lossy().replace('\\', "/"),
        "run_manifest_ref": "target/ocp/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report("lsp/runtime_resilience_report.json", &report);
}
