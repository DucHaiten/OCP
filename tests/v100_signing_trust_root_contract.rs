use std::fs;

use serde_json::{json, Value as JsonValue};

#[path = "v100_gate_a_common.rs"]
mod v100;

#[test]
fn v100_signing_trust_root_contract() {
    v100::ensure_run_manifest();

    let trust_path = v100::signing_trust_root_path();
    let trust = v100::read_json(&trust_path);
    assert_eq!(
        trust
            .get("contract_id")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "v1.signing_trust_root"
    );

    let keys = trust
        .get("trust_root_public_keys")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(!keys.is_empty(), "trust_root_public_keys must not be empty");
    let key_ids = keys
        .iter()
        .map(|row| {
            row.get("key_id")
                .and_then(JsonValue::as_str)
                .unwrap_or_else(|| panic!("missing key_id in trust_root_public_keys entry: {row}"))
                .to_string()
        })
        .collect::<Vec<String>>();
    for row in &keys {
        let fingerprint = row
            .get("fingerprint")
            .and_then(JsonValue::as_str)
            .unwrap_or("");
        assert!(
            !fingerprint.trim().is_empty(),
            "fingerprint must be non-empty for trust root key"
        );
    }

    let epoch = trust
        .get("trust_epoch_policy")
        .and_then(JsonValue::as_object)
        .expect("trust_epoch_policy object");
    let min = epoch.get("min").and_then(JsonValue::as_u64).unwrap_or(0);
    let max = epoch.get("max").and_then(JsonValue::as_u64).unwrap_or(0);
    let current = epoch
        .get("current")
        .and_then(JsonValue::as_u64)
        .unwrap_or(0);
    assert!(min > 0, "trust_epoch_policy.min must be > 0");
    assert!(max >= min, "trust_epoch_policy.max must be >= min");
    assert!(
        current >= min && current <= max,
        "trust_epoch_policy.current must be in range"
    );

    let rotation_policy = trust
        .get("key_rotation_policy")
        .and_then(JsonValue::as_str)
        .unwrap_or("");
    assert!(
        !rotation_policy.trim().is_empty(),
        "key_rotation_policy must be non-empty"
    );

    let trust_toml =
        fs::read_to_string(v100::repo_root().join("trust.toml")).expect("read trust.toml");
    let signer_in_toml = key_ids.iter().any(|key_id| trust_toml.contains(key_id));
    assert!(
        signer_in_toml,
        "trust.toml must contain at least one key_id from signing_trust_root contract"
    );

    let report = json!({
        "schema": "ocl.w100.signing_trust_root_report.v1",
        "status": "PASS",
        "trust_contract_path": trust_path.to_string_lossy().replace('\\', "/"),
        "key_ids": key_ids,
        "trust_epoch_policy": {
            "min": min,
            "max": max,
            "current": current
        },
        "key_rotation_policy": rotation_policy,
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100::run_manifest_sha256()
    });
    v100::write_report("contracts/signing_trust_root_report.json", &report);
}
