#[path = "w17_gate_e_common.rs"]
mod w17;

use ocp_sdk::{evaluate_pack_boundary_v17, W17_PACK_BOUNDARY_WASI_V1};

#[test]
fn v17_pack_sandbox_boundary_rejects_unknown_boundary() {
    let decision = evaluate_pack_boundary_v17("pack_boundary_unknown_v1", &[], false);
    assert!(!decision.allowed);
    assert_eq!(decision.reason_code, "RC-PACK-BOUNDARY-UNKNOWN");
    w17::write_gate_e_report();
}

#[test]
fn v17_pack_sandbox_boundary_rejects_direct_host_io() {
    let decision = evaluate_pack_boundary_v17(W17_PACK_BOUNDARY_WASI_V1, &[], true);
    assert!(!decision.allowed);
    assert_eq!(decision.reason_code, "RC-PACK-BOUNDARY-NO-NAKED-IO");
    w17::write_gate_e_report();
}
