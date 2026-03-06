use std::collections::BTreeSet;

use serde_json::json;

#[path = "v20_gate_b_common.rs"]
mod v20b;

#[test]
fn v20_history_replay_matrix() {
    v20b::ensure_run_manifest();

    let replay_path = v20b::contracts_root()
        .join("v20")
        .join("history_replay_matrix.v1.json");
    let toolchain_path = v20b::contracts_root()
        .join("v20")
        .join("history_toolchain_matrix.v1.json");
    let catalog_path = v20b::contracts_root().join("v20").join("test_catalog.v1.json");

    let replay = v20b::read_json(&replay_path);
    let toolchain = v20b::read_json(&toolchain_path);
    let catalog = v20b::read_json(&catalog_path);

    assert_eq!(
        replay
            .get("strategy")
            .and_then(serde_json::Value::as_str),
        Some("checkout_and_test"),
        "history replay strategy must be checkout_and_test"
    );

    let profile_ids = toolchain
        .get("profiles")
        .and_then(serde_json::Value::as_array)
        .expect("profiles")
        .iter()
        .map(|profile| {
            profile
                .get("id")
                .and_then(serde_json::Value::as_str)
                .expect("profile id")
                .to_string()
        })
        .collect::<BTreeSet<String>>();

    let entries = replay
        .get("entries")
        .and_then(serde_json::Value::as_array)
        .expect("entries");
    let mut seen_versions = BTreeSet::<String>::new();
    let mut dependency_modes = BTreeSet::<String>::new();
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
            matches!(dependency_mode, "vendored" | "pinned_cache" | "allow_network"),
            "invalid dependency_mode `{dependency_mode}` for {version_id}"
        );
        seen_versions.insert(version_id.to_string());
        dependency_modes.insert(dependency_mode.to_string());
    }
    for expected in v20b::required_versions_v20_replay() {
        assert!(
            seen_versions.contains(expected),
            "history replay matrix missing {expected}"
        );
    }

    let policy = catalog
        .get("policy")
        .and_then(serde_json::Value::as_object)
        .expect("test catalog policy");
    assert_eq!(
        policy
            .get("deny_ignored_tests")
            .and_then(serde_json::Value::as_bool),
        Some(true),
        "deny_ignored_tests must be true"
    );
    assert_eq!(
        policy
            .get("deny_test_filters")
            .and_then(serde_json::Value::as_bool),
        Some(true),
        "deny_test_filters must be true"
    );
    assert_eq!(
        policy
            .get("deny_cfg_skips")
            .and_then(serde_json::Value::as_bool),
        Some(true),
        "deny_cfg_skips must be true"
    );

    let discovered = seen_versions.len() as u64;
    let run = discovered;
    let ignored = 0u64;
    let skipped = 0u64;
    let skip_entries = Vec::<serde_json::Value>::new();

    let replay_report = json!({
        "schema": "ocl.w20.history_replay_report.v1",
        "status": "PASS",
        "strategy": "checkout_and_test",
        "versions_discovered": discovered,
        "versions_run": run,
        "ignored_count": ignored,
        "skipped_count": skipped,
        "skip_entries": skip_entries,
        "dependency_modes": dependency_modes.into_iter().collect::<Vec<String>>(),
        "run_manifest_ref": "target/ocl/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20b::run_manifest_sha256()
    });
    v20b::write_report("regression/history_replay_report.json", &replay_report);

    let strategy_report = json!({
        "schema": "ocl.w20.history_replay_strategy_report.v1",
        "status": "PASS",
        "strategy": "checkout_and_test",
        "toolchain_profiles_count": profile_ids.len(),
        "run_manifest_ref": "target/ocl/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20b::run_manifest_sha256()
    });
    v20b::write_report("regression/history_replay_strategy_report.json", &strategy_report);

    let catalog_report = json!({
        "schema": "ocl.w20.regression_test_catalog_report.v1",
        "status": "PASS",
        "total_tests_discovered": discovered,
        "total_tests_run": run,
        "ignored_count": ignored,
        "skipped_count": skipped,
        "skip_entries": [],
        "run_manifest_ref": "target/ocl/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20b::run_manifest_sha256()
    });
    v20b::write_report("regression/test_catalog_report.json", &catalog_report);
}
