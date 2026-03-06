#[path = "w17_gate_e_common.rs"]
mod w17;

use ocl_sdk::{evaluate_pack_boundary_v17, W17_PACK_BOUNDARY_WASI_V1};

#[test]
fn v17_pack_boundary_wasi_accepts_wasi_scoped_capabilities() {
    let decision = evaluate_pack_boundary_v17(
        W17_PACK_BOUNDARY_WASI_V1,
        &["wasi.fs.read".to_string(), "wasi.net.http".to_string()],
        false,
    );
    assert!(decision.allowed);
    assert_eq!(decision.reason_code, "RC-PACK-BOUNDARY-WASI-OK");
    w17::write_gate_e_report();
}

#[test]
fn v17_pack_boundary_wasi_rejects_native_capability_labels() {
    let decision = evaluate_pack_boundary_v17(
        W17_PACK_BOUNDARY_WASI_V1,
        &["native.proc.exec".to_string()],
        false,
    );
    assert!(!decision.allowed);
    assert_eq!(decision.reason_code, "RC-PACK-BOUNDARY-WASI-CAP-MISMATCH");
    w17::write_gate_e_report();
}
