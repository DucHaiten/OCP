use std::fs;

use serde_json::Value as JsonValue;

#[path = "w17_gate_b_cli_common.rs"]
mod common;

fn seed_and_upgrade(artifact: &std::path::Path) {
    let cassette_dir = artifact.join("cassette");
    fs::create_dir_all(&cassette_dir).expect("create cassette");
    fs::write(
        cassette_dir.join("cassette.jsonl"),
        "{\"id\":\"e1\",\"type\":\"wallclock\",\"unix_ms\":1}\n",
    )
    .expect("write cassette.jsonl");
    fs::write(
        cassette_dir.join("cassette_index.json"),
        "{\"call_id_to_entry_id\":{}}",
    )
    .expect("write cassette_index.json");
    fs::write(
        cassette_dir.join("cassette_meta.toml"),
        "mode = \"record\"\n",
    )
    .expect("write cassette_meta.toml");

    let artifact_s = artifact.to_string_lossy().to_string();
    let out = common::run_ocp_cli(&["cassette", "upgrade", &artifact_s, "--apply", "--json"]);
    common::assert_success(&out);
}

#[test]
fn cassette_prune_plan_and_apply_remove_orphan_blocks() {
    let root = common::temp_dir("cassette_prune");
    let artifact = common::ensure_artifact_layout(&root);
    seed_and_upgrade(&artifact);

    let orphan_path = artifact
        .join("cassette")
        .join("blocks")
        .join("deadbeef.json");
    fs::write(&orphan_path, "{}").expect("write orphan block");
    assert!(orphan_path.exists(), "orphan block must exist before prune");

    let artifact_s = artifact.to_string_lossy().to_string();
    let plan = common::run_ocp_cli(&["cassette", "prune", &artifact_s, "--plan", "--json"]);
    let plan_json = common::assert_success(&plan);
    let plan_value: JsonValue = serde_json::from_str(&plan_json).expect("plan json");
    let orphans = plan_value
        .get("orphan_blocks")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(
        orphans.iter().any(|v| v.as_str() == Some("deadbeef")),
        "prune plan must include injected orphan block"
    );

    let apply = common::run_ocp_cli(&["cassette", "prune", &artifact_s, "--apply", "--json"]);
    let apply_json = common::assert_success(&apply);
    let apply_value: JsonValue = serde_json::from_str(&apply_json).expect("apply json");
    let applied = apply_value
        .get("applied_blocks")
        .and_then(JsonValue::as_u64)
        .unwrap_or(0);
    assert!(applied >= 1, "expected at least one applied prune block");
    assert!(
        !orphan_path.exists(),
        "orphan block must be removed after prune --apply"
    );

    let stats = common::run_ocp_cli(&["cassette", "stats", &artifact_s, "--json"]);
    let stats_json = common::assert_success(&stats);
    let stats_value: JsonValue = serde_json::from_str(&stats_json).expect("stats json");
    let entries = stats_value
        .get("entries")
        .and_then(JsonValue::as_u64)
        .unwrap_or(0);
    assert!(entries >= 1, "cassette stats must report entries");
    assert!(
        root.join("target")
            .join("ocp")
            .join("w17")
            .join("cassette")
            .join("cassette_operability_report.json")
            .exists(),
        "missing cassette_operability_report.json"
    );
}
