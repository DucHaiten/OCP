#[path = "w17_gate_e_common.rs"]
mod w17;

use ocl_sdk::evaluate_pack_trust_policy_v17;

#[test]
fn v17_pack_trust_policy_locked_v071_requires_full_chain() {
    let missing_attestation =
        evaluate_pack_trust_policy_v17("locked_v071", true, false, true, true, true);
    assert!(!missing_attestation.allowed);
    assert_eq!(
        missing_attestation.reason_code,
        "RC-PACK-ATTESTATION-REQUIRED"
    );

    let missing_trust =
        evaluate_pack_trust_policy_v17("locked_v071", true, true, false, true, true);
    assert!(!missing_trust.allowed);
    assert_eq!(missing_trust.reason_code, "RC-PACK-TRUST-REQUIRED");

    w17::write_gate_e_report();
}

#[test]
fn v17_pack_trust_policy_locked_v071_allows_only_when_full_chain_present() {
    let decision = evaluate_pack_trust_policy_v17("locked_v071", true, true, true, true, true);
    assert!(decision.allowed);
    assert!(!decision.requires_audit_marker);
    assert_eq!(decision.reason_code, "RC-PACK-TRUST-OK");
    w17::write_gate_e_report();
}

#[test]
fn v17_pack_trust_policy_quarantine_keeps_audit_marker() {
    let decision = evaluate_pack_trust_policy_v17("quarantine", false, false, false, false, false);
    assert!(decision.allowed);
    assert!(decision.requires_audit_marker);
    assert_eq!(decision.reason_code, "RC-PACK-TRUST-QUARANTINE-AUDIT");
    w17::write_gate_e_report();
}
