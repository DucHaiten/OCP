#[path = "w17_gate_e_common.rs"]
mod w17;

use ocl_sdk::{evaluate_pack_boundary_v17, W17_PACK_BOUNDARY_NATIVE_CAP_V1};

#[test]
fn v17_pack_boundary_native_cap_accepts_native_scoped_capabilities() {
    let decision = evaluate_pack_boundary_v17(
        W17_PACK_BOUNDARY_NATIVE_CAP_V1,
        &["native.fs.read".to_string(), "native.proc.exec".to_string()],
        false,
    );
    assert!(decision.allowed);
    assert_eq!(decision.reason_code, "RC-PACK-BOUNDARY-NATIVE-OK");
    w17::write_gate_e_report();
}

#[test]
fn v17_pack_boundary_native_cap_rejects_wasi_capability_labels() {
    let decision = evaluate_pack_boundary_v17(
        W17_PACK_BOUNDARY_NATIVE_CAP_V1,
        &["wasi.net.http".to_string()],
        false,
    );
    assert!(!decision.allowed);
    assert_eq!(decision.reason_code, "RC-PACK-BOUNDARY-NATIVE-CAP-MISMATCH");
    w17::write_gate_e_report();
}
