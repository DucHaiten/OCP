use std::env;
use std::fs;

use serde_json::json;

#[path = "v20_gate_b_common.rs"]
mod v20b;

#[test]
fn v20_cross_platform_signature() {
    v20b::ensure_run_manifest();

    let replay_path = v20b::contracts_root()
        .join("v20")
        .join("history_replay_matrix.v1.json");
    let toolchain_path = v20b::contracts_root()
        .join("v20")
        .join("history_toolchain_matrix.v1.json");
    let required_path = v20b::contracts_root()
        .join("v20")
        .join("required_contracts_v20.v1.json");
    let replay_hash = {
        let value = v20b::read_json(&replay_path);
        let canonical = v20b::canonical_json_string(&value);
        v20b::sha256_hex_bytes(canonical.as_bytes())
    };
    let toolchain_hash = {
        let value = v20b::read_json(&toolchain_path);
        let canonical = v20b::canonical_json_string(&value);
        v20b::sha256_hex_bytes(canonical.as_bytes())
    };
    let required_hash = {
        let value = v20b::read_json(&required_path);
        let canonical = v20b::canonical_json_string(&value);
        v20b::sha256_hex_bytes(canonical.as_bytes())
    };
    let seed = format!("{}|{}|{}", replay_hash, toolchain_hash, required_hash);
    let canonical_signature = v20b::sha256_hex_bytes(seed.as_bytes());

    let current_profile = v20b::current_os_profile();
    assert_ne!(
        current_profile, "unsupported",
        "current OS/arch is outside supported profile for v20 cross-platform check"
    );

    let current_report = json!({
        "schema": "ocp.w20.cross_platform_signature_report.v1",
        "status": "PASS",
        "os_profile": current_profile,
        "signature": canonical_signature,
        "source_seed_hash": v20b::sha256_hex_bytes(seed.as_bytes()),
        "run_manifest_ref": "target/ocp/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20b::run_manifest_sha256()
    });
    let current_path = v20b::w20_target_root()
        .join("regression")
        .join("os")
        .join(current_profile)
        .join("cross_platform_signature_report.json");
    if let Some(parent) = current_path.parent() {
        fs::create_dir_all(parent).expect("create per-os report dir");
    }
    v20b::write_json_pretty(&current_path, &current_report);

    let mode = env::var("W20_CROSS_PLATFORM_MODE").unwrap_or_else(|_| "strict".to_string());
    assert!(
        matches!(mode.as_str(), "strict" | "per_os" | "aggregate"),
        "invalid W20_CROSS_PLATFORM_MODE: {} (expected strict|per_os|aggregate)",
        mode
    );

    if mode == "per_os" {
        return;
    }

    let mut seen = Vec::<serde_json::Value>::new();
    let mut signatures = Vec::<String>::new();
    let mut missing = Vec::<String>::new();
    for profile in v20b::expected_os_profiles() {
        let path = v20b::w20_target_root()
            .join("regression")
            .join("os")
            .join(profile)
            .join("cross_platform_signature_report.json");
        if !path.exists() {
            missing.push(profile.to_string());
            continue;
        }
        let value = v20b::read_json(&path);
        let signature = value
            .get("signature")
            .and_then(serde_json::Value::as_str)
            .expect("signature in per-os report")
            .to_string();
        signatures.push(signature.clone());
        seen.push(json!({
            "os_profile": profile,
            "signature": signature,
            "report_path": path.to_string_lossy().replace('\\', "/")
        }));
    }

    let aggregate = json!({
        "schema": "ocp.w20.cross_platform_signature_aggregate_report.v1",
        "mode": mode,
        "status": if missing.is_empty() { "PASS" } else { "INCOMPLETE" },
        "required_profiles": v20b::expected_os_profiles(),
        "missing_profiles": missing,
        "items": seen,
        "run_manifest_ref": "target/ocp/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20b::run_manifest_sha256()
    });
    v20b::write_report(
        "regression/cross_platform_signature_aggregate_report.json",
        &aggregate,
    );

    if !aggregate["missing_profiles"]
        .as_array()
        .expect("missing_profiles")
        .is_empty()
    {
        panic!(
            "missing per-OS cross-platform signature reports for: {}",
            aggregate["missing_profiles"]
                .as_array()
                .expect("missing_profiles array")
                .iter()
                .map(|item| item.as_str().unwrap_or("?"))
                .collect::<Vec<&str>>()
                .join(", ")
        );
    }

    let first = signatures
        .first()
        .expect("at least one signature")
        .to_string();
    let mismatch = signatures.iter().any(|item| item != &first);
    assert!(!mismatch, "cross-platform signatures must be identical");
}
