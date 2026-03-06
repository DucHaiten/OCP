use std::collections::BTreeSet;

use serde_json::json;

#[path = "v20_gate_b_common.rs"]
mod v20b;

#[test]
fn v20_history_toolchain_matrix() {
    v20b::ensure_run_manifest();

    let matrix_path = v20b::contracts_root()
        .join("v20")
        .join("history_toolchain_matrix.v1.json");
    let replay_path = v20b::contracts_root()
        .join("v20")
        .join("history_replay_matrix.v1.json");
    let matrix = v20b::read_json(&matrix_path);
    let replay = v20b::read_json(&replay_path);

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
        for key in ["rustc", "cargo", "node", "pnpm"] {
            assert!(
                profile.get(key).and_then(serde_json::Value::as_str).is_some(),
                "profile `{id}` missing `{key}`"
            );
        }
        profile_ids.insert(id.to_string());
    }

    let entries = replay
        .get("entries")
        .and_then(serde_json::Value::as_array)
        .expect("history replay entries");
    for entry in entries {
        let version_id = entry
            .get("version_id")
            .and_then(serde_json::Value::as_str)
            .expect("version_id");
        let profile = entry
            .get("toolchain_profile")
            .and_then(serde_json::Value::as_str)
            .expect("toolchain_profile");
        assert!(
            profile_ids.contains(profile),
            "unknown toolchain profile `{profile}` for {version_id}"
        );
    }

    let report = json!({
        "schema": "ocl.w20.history_toolchain_matrix_report.v1",
        "status": "PASS",
        "profile_count": profile_ids.len(),
        "replay_entries_count": entries.len(),
        "matrix_hash_sha256": v20b::sha256_hex_file(&matrix_path),
        "run_manifest_ref": "target/ocl/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20b::run_manifest_sha256()
    });
    v20b::write_report("regression/history_toolchain_matrix_report.json", &report);
}
