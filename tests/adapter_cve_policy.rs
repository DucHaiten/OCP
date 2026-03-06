#[path = "w17_gate_e_common.rs"]
mod w17;

use ocl_sdk::{evaluate_adapter_cve_policy_v17, AdapterCveSeverityV17};

#[test]
fn v17_adapter_cve_policy_locked_v071_blocks_unpatched_critical() {
    let decision = evaluate_adapter_cve_policy_v17(
        "locked_v071",
        AdapterCveSeverityV17::Critical,
        false,
        true,
        true,
    );
    assert!(!decision.allowed);
    assert!(decision.requires_audit_marker);
    assert_eq!(decision.reason_code, "RC-CVE-BLOCK-STRICT");
    w17::write_gate_e_report();
}

#[test]
fn v17_adapter_cve_policy_locked_v071_requires_sbom() {
    let decision = evaluate_adapter_cve_policy_v17(
        "locked_v071",
        AdapterCveSeverityV17::Low,
        true,
        false,
        false,
    );
    assert!(!decision.allowed);
    assert_eq!(decision.reason_code, "RC-CVE-SBOM-REQUIRED");
    w17::write_gate_e_report();
}

#[test]
fn v17_adapter_cve_policy_quarantine_allows_with_audit_marker() {
    let decision = evaluate_adapter_cve_policy_v17(
        "quarantine",
        AdapterCveSeverityV17::High,
        false,
        true,
        true,
    );
    assert!(decision.allowed);
    assert!(decision.requires_audit_marker);
    assert_eq!(decision.reason_code, "RC-CVE-QUARANTINE-AUDIT");
    w17::write_gate_e_report();
}
