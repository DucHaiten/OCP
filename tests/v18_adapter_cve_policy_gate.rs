use serde_json::json;

use ocp_sdk::{evaluate_adapter_cve_policy_v17, AdapterCveSeverityV17};

#[path = "v18_gate_e_common.rs"]
mod common;

#[test]
fn v18_adapter_cve_policy_gate_blocks_strict_lane_and_audits_quarantine() {
    common::ensure_run_manifest();

    let strict_critical = evaluate_adapter_cve_policy_v17(
        "locked_v071",
        AdapterCveSeverityV17::Critical,
        false,
        true,
        true,
    );
    assert!(!strict_critical.allowed);
    assert_eq!(strict_critical.reason_code, "RC-CVE-BLOCK-STRICT");

    let strict_missing_sbom = evaluate_adapter_cve_policy_v17(
        "locked_v071",
        AdapterCveSeverityV17::Low,
        true,
        false,
        false,
    );
    assert!(!strict_missing_sbom.allowed);
    assert_eq!(strict_missing_sbom.reason_code, "RC-CVE-SBOM-REQUIRED");

    let quarantine = evaluate_adapter_cve_policy_v17(
        "quarantine",
        AdapterCveSeverityV17::High,
        false,
        true,
        true,
    );
    assert!(quarantine.allowed);
    assert!(quarantine.requires_audit_marker);
    assert_eq!(quarantine.reason_code, "RC-CVE-QUARANTINE-AUDIT");

    common::merge_pack_shipproof_section(
        "adapter_cve_policy_gate",
        json!({
            "status": "PASS",
            "strict_critical_reason": strict_critical.reason_code,
            "strict_missing_sbom_reason": strict_missing_sbom.reason_code,
            "quarantine_reason": quarantine.reason_code
        }),
    );
}
