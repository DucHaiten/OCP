use std::collections::BTreeSet;

use serde_json::json;

#[path = "v20_gate_b_common.rs"]
mod v20b;

#[test]
fn v20_regression_snapshot_diff() {
    v20b::ensure_run_manifest();

    let replay_path = v20b::contracts_root()
        .join("v20")
        .join("history_replay_matrix.v1.json");
    let replay = v20b::read_json(&replay_path);
    let entries = replay
        .get("entries")
        .and_then(serde_json::Value::as_array)
        .expect("history replay entries");

    let mut missing_plan_files = Vec::<String>::new();
    let mut replay_versions = BTreeSet::<String>::new();
    let mut replay_hashes = Vec::<serde_json::Value>::new();
    for entry in entries {
        let version_id = entry
            .get("version_id")
            .and_then(serde_json::Value::as_str)
            .expect("version_id");
        replay_versions.insert(version_id.to_string());
        let Some(plan_rel) = v20b::plan_file_for_version(version_id) else {
            missing_plan_files.push(format!("no plan mapping for {version_id}"));
            continue;
        };
        let plan_path = v20b::repo_root().join(plan_rel);
        if !plan_path.exists() {
            missing_plan_files.push(plan_rel.to_string());
            continue;
        }
        replay_hashes.push(json!({
            "version_id": version_id,
            "plan_path": plan_rel,
            "sha256": v20b::sha256_hex_file(&plan_path)
        }));
    }

    assert!(
        missing_plan_files.is_empty(),
        "missing plan files in replay matrix: {}",
        missing_plan_files.join(", ")
    );

    let legacy_index_path = v20b::contracts_root()
        .join("history")
        .join("evidence_index.v1.json");
    let mut legacy_versions = BTreeSet::<String>::new();
    if legacy_index_path.exists() {
        let legacy = v20b::read_json(&legacy_index_path);
        if let Some(items) = legacy.get("entries").and_then(serde_json::Value::as_array) {
            for item in items {
                if let Some(version) = item.get("version_id").and_then(serde_json::Value::as_str) {
                    legacy_versions.insert(version.to_string());
                }
            }
        }
    }

    let missing_in_legacy = replay_versions
        .iter()
        .filter(|version| !legacy_versions.contains(*version))
        .cloned()
        .collect::<Vec<String>>();

    let report = json!({
        "schema": "ocp.w20.regression_diff_report.v1",
        "status": "PASS",
        "replay_versions_count": replay_versions.len(),
        "legacy_versions_count": legacy_versions.len(),
        "missing_in_legacy_index": missing_in_legacy,
        "snapshot_hashes": replay_hashes,
        "run_manifest_ref": "target/ocp/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20b::run_manifest_sha256()
    });
    v20b::write_report("regression/regression_diff_report.json", &report);
}
