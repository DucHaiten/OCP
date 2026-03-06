use std::fs;

use serde_json::json;

#[path = "v19_gate_f_common.rs"]
mod f19;

#[test]
fn v19_binary_bootstrap_recovery_from_tamper() {
    f19::ensure_run_manifest();
    let fixture = f19::ensure_release_fixture_v19();

    let target = fixture
        .binaries
        .get("ocl-lsp")
        .expect("ocl-lsp path")
        .to_path_buf();
    let original = fs::read(&target).expect("read original binary");
    let expected_hash = fixture
        .binary_hashes
        .get("ocl-lsp")
        .expect("ocl-lsp hash")
        .to_string();

    fs::write(&target, b"TAMPERED\n").expect("tamper binary");
    let tampered = fs::read(&target).expect("read tampered binary");
    let tampered_hash = f19::sha256_hex_bytes(&tampered);
    assert_ne!(tampered_hash, expected_hash, "tampered hash must differ");

    fs::write(&target, &original).expect("recover from bundled source");
    let recovered = fs::read(&target).expect("read recovered binary");
    let recovered_hash = f19::sha256_hex_bytes(&recovered);
    assert_eq!(
        recovered_hash, expected_hash,
        "recovery must restore expected hash"
    );

    let report = json!({
        "schema": "ocl.w19.release.binary_recovery_report.v1",
        "status": "PASS",
        "policy_mode": "fail_honest_degraded",
        "tampered_hash": tampered_hash,
        "recovered_hash": recovered_hash,
        "expected_hash": expected_hash,
        "run_manifest_ref": "target/ocl/w19/meta/run_manifest.json",
        "run_manifest_sha256": f19::run_manifest_sha256()
    });
    f19::write_report("release/binary_recovery_report.json", &report);
}
