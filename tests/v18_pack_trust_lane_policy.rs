use serde_json::json;

use ocl_sdk::evaluate_pack_trust_policy_v17;

#[path = "v18_gate_e_common.rs"]
mod common;

#[test]
fn v18_pack_trust_lane_policy_is_fail_honest_in_locked_and_audited_in_quarantine() {
    common::ensure_run_manifest();

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

    let locked_ok = evaluate_pack_trust_policy_v17("locked_v071", true, true, true, true, true);
    assert!(locked_ok.allowed);
    assert!(!locked_ok.requires_audit_marker);
    assert_eq!(locked_ok.reason_code, "RC-PACK-TRUST-OK");

    let quarantine =
        evaluate_pack_trust_policy_v17("quarantine", false, false, false, false, false);
    assert!(quarantine.allowed);
    assert!(quarantine.requires_audit_marker);
    assert_eq!(quarantine.reason_code, "RC-PACK-TRUST-QUARANTINE-AUDIT");

    common::merge_pack_shipproof_section(
        "pack_trust_lane_policy",
        json!({
            "status": "PASS",
            "locked_missing_attestation_reason": missing_attestation.reason_code,
            "locked_missing_trust_reason": missing_trust.reason_code,
            "locked_ok_reason": locked_ok.reason_code,
            "quarantine_reason": quarantine.reason_code
        }),
    );
}
