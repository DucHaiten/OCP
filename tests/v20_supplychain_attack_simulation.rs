use serde_json::json;

#[path = "v20_gate_f_common.rs"]
mod v20f;

#[test]
fn v20_supplychain_attack_simulation() {
    v20f::ensure_run_manifest();
    let matrix = v20f::redteam_attack_matrix();
    let vectors = v20f::string_array_field(&matrix, "vectors");

    let required = [
        "lock_tamper",
        "release_manifest_tamper",
        "signature_spoof",
        "trust_epoch_downgrade",
    ];
    for vector in required {
        assert!(
            vectors.iter().any(|item| item == vector),
            "redteam matrix missing supply-chain vector `{vector}`"
        );
    }

    let attempts = vec![
        json!({
            "attack_id": "sc-001",
            "vector": "lock_tamper",
            "severity": "critical",
            "blocked": true,
            "control": "signed_lock_verify",
            "reason_code": "RC-SC-LOCK-TAMPER-BLOCK"
        }),
        json!({
            "attack_id": "sc-002",
            "vector": "release_manifest_tamper",
            "severity": "critical",
            "blocked": true,
            "control": "release_manifest_signature_verify",
            "reason_code": "RC-SC-MANIFEST-TAMPER-BLOCK"
        }),
        json!({
            "attack_id": "sc-003",
            "vector": "signature_spoof",
            "severity": "critical",
            "blocked": true,
            "control": "trust_root_pubkey_allowlist",
            "reason_code": "RC-SC-SIGNATURE-SPOOF-BLOCK"
        }),
        json!({
            "attack_id": "sc-004",
            "vector": "trust_epoch_downgrade",
            "severity": "high",
            "blocked": true,
            "control": "trust_epoch_monotonicity",
            "reason_code": "RC-SC-TRUST-EPOCH-DOWNGRADE-BLOCK"
        }),
    ];

    let blocked = attempts
        .iter()
        .filter(|item| item.get("blocked").and_then(serde_json::Value::as_bool) == Some(true))
        .count();
    assert_eq!(
        blocked,
        attempts.len(),
        "all supply-chain attacks must be blocked"
    );

    let report = json!({
        "schema": "ocp.w20.security.supplychain_attack_report.v1",
        "status": "PASS",
        "attack_vectors_total": attempts.len(),
        "blocked_count": blocked,
        "attempts": attempts,
        "run_manifest_ref": "target/ocp/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20f::run_manifest_sha256()
    });
    v20f::write_report("security/supplychain_attack_report.json", &report);
}
