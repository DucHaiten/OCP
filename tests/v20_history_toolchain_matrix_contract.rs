use std::collections::BTreeSet;

use serde_json::json;

#[path = "v20_gate_a_common.rs"]
mod v20;

#[test]
fn v20_history_toolchain_matrix_contract() {
    v20::ensure_run_manifest();

    let matrix_path = v20::contracts_root()
        .join("v20")
        .join("history_toolchain_matrix.v1.json");
    let matrix = v20::read_json(&matrix_path);
    assert_eq!(
        matrix
            .get("contract_id")
            .and_then(serde_json::Value::as_str),
        Some("v20.history_toolchain_matrix"),
        "history_toolchain_matrix contract_id must be stable"
    );
    let profiles = matrix
        .get("profiles")
        .and_then(serde_json::Value::as_array)
        .expect("profiles");
    assert!(!profiles.is_empty(), "toolchain profiles must not be empty");

    let mut profile_ids = BTreeSet::<String>::new();
    for profile in profiles {
        let id = profile
            .get("id")
            .and_then(serde_json::Value::as_str)
            .expect("profile id");
        assert!(
            profile
                .get("rustc")
                .and_then(serde_json::Value::as_str)
                .is_some(),
            "profile `{id}` missing rustc"
        );
        assert!(
            profile
                .get("cargo")
                .and_then(serde_json::Value::as_str)
                .is_some(),
            "profile `{id}` missing cargo"
        );
        assert!(
            profile
                .get("node")
                .and_then(serde_json::Value::as_str)
                .is_some(),
            "profile `{id}` missing node"
        );
        assert!(
            profile
                .get("pnpm")
                .and_then(serde_json::Value::as_str)
                .is_some(),
            "profile `{id}` missing pnpm"
        );
        profile_ids.insert(id.to_string());
    }

    let replay_path = v20::contracts_root()
        .join("v20")
        .join("history_replay_matrix.v1.json");
    let replay = v20::read_json(&replay_path);
    let entries = replay
        .get("entries")
        .and_then(serde_json::Value::as_array)
        .expect("history replay entries");
    assert!(
        entries.len() >= 19,
        "history replay matrix must include at least v0.1..v0.19"
    );

    let mut versions = BTreeSet::<String>::new();
    for entry in entries {
        let version_id = entry
            .get("version_id")
            .and_then(serde_json::Value::as_str)
            .expect("version_id");
        let profile = entry
            .get("toolchain_profile")
            .and_then(serde_json::Value::as_str)
            .expect("toolchain_profile");
        let dependency_mode = entry
            .get("dependency_mode")
            .and_then(serde_json::Value::as_str)
            .expect("dependency_mode");
        assert!(
            profile_ids.contains(profile),
            "unknown toolchain_profile `{profile}` for {version_id}"
        );
        assert!(
            matches!(
                dependency_mode,
                "vendored" | "pinned_cache" | "allow_network"
            ),
            "invalid dependency_mode `{dependency_mode}` for {version_id}"
        );
        versions.insert(version_id.to_string());
    }
    for required in [
        "v0.1", "v0.2", "v0.3", "v0.4", "v0.5", "v0.6", "v0.7.2", "v0.7.3", "v0.8", "v0.9",
        "v0.10", "v0.11", "v0.12", "v0.13", "v0.14", "v0.15", "v0.16", "v0.17", "v0.18", "v0.19",
    ] {
        assert!(
            versions.contains(required),
            "history replay matrix missing {required}"
        );
    }

    let report = json!({
        "schema": "ocl.w20.history_toolchain_matrix_report.v1",
        "status": "PASS",
        "profile_count": profiles.len(),
        "version_count": versions.len(),
        "run_manifest_ref": "target/ocl/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20::run_manifest_sha256()
    });
    v20::write_report("contracts/history_toolchain_matrix_report.json", &report);
}
