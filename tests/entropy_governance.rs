#[path = "w17_gate_c_common.rs"]
mod w17;

use ocp::ocp::{evaluate_entropy_policy, EntropySource};

#[test]
fn locked_lane_blocks_host_entropy_without_governed_path() {
    let decision = evaluate_entropy_policy("locked_v071", EntropySource::HostRandom, false);
    assert!(!decision.allowed, "locked lane must deny host entropy");
    assert!(
        decision.requires_audit_marker,
        "locked lane deny must leave audit marker"
    );
    assert_eq!(decision.reason_code, "RC-ENTROPY-BLOCKED-LOCKED");
    w17::write_gate_c_report();
}

#[test]
fn locked_lane_allows_governed_seeded_rng() {
    let decision = evaluate_entropy_policy("locked_v071", EntropySource::GovernedSeededRng, true);
    assert!(decision.allowed, "governed rng must be allowed");
    assert!(
        !decision.requires_audit_marker,
        "governed rng should not require degradation marker"
    );
    assert_eq!(decision.reason_code, "RC-ENTROPY-GOVERNED");
    w17::write_gate_c_report();
}

#[test]
fn quarantine_lane_allows_entropy_with_audit_marker() {
    let decision = evaluate_entropy_policy("quarantine", EntropySource::SystemTime, false);
    assert!(decision.allowed, "quarantine lane may allow entropy");
    assert!(
        decision.requires_audit_marker,
        "quarantine entropy must be audited"
    );
    assert_eq!(decision.reason_code, "RC-ENTROPY-AUDIT");
    w17::write_gate_c_report();
}

#[test]
fn unknown_lane_is_fail_honest() {
    let decision = evaluate_entropy_policy("custom_lane", EntropySource::Uuid, false);
    assert!(!decision.allowed, "unknown lane must fail-honest");
    assert!(
        decision.requires_audit_marker,
        "unknown lane deny should be auditable"
    );
    assert_eq!(decision.reason_code, "RC-LANE-UNSUPPORTED");
    w17::write_gate_c_report();
}
