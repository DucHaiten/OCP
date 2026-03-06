use std::fs;

use serde_json::{json, Value as JsonValue};

#[path = "v18_gate_d_common.rs"]
mod common;

fn seed_and_upgrade(artifact: &std::path::Path) {
    common::seed_v08_cassette_lines(
        artifact,
        &["{\"id\":\"e1\",\"type\":\"wallclock\",\"unix_ms\":1}"],
    );
    let artifact_s = artifact.to_string_lossy().to_string();
    let out = common::run_ocl_cli(&["cassette", "upgrade", &artifact_s, "--apply", "--json"]);
    let _ = common::assert_success(&out);
}

#[test]
fn v18_cassette_prune_plan_apply_removes_orphans_and_keeps_indexed_blocks() {
    common::ensure_run_manifest();
    let root = common::temp_dir("prune_plan_apply");
    let artifact = common::ensure_artifact_layout(&root);
    seed_and_upgrade(&artifact);

    let orphan_path = artifact
        .join("cassette")
        .join("blocks")
        .join("deadbeef.json");
    fs::write(&orphan_path, "{}").expect("write orphan block");
    assert!(orphan_path.exists(), "orphan block must exist before prune");

    let artifact_s = artifact.to_string_lossy().to_string();
    let plan = common::run_ocl_cli(&["cassette", "prune", &artifact_s, "--plan", "--json"]);
    let plan_stdout = common::assert_success(&plan);
    let plan_json: JsonValue = serde_json::from_str(&plan_stdout).expect("parse prune plan");
    let orphan_blocks = plan_json
        .get("orphan_blocks")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(
        orphan_blocks
            .iter()
            .any(|item| item.as_str() == Some("deadbeef")),
        "prune plan must include injected orphan block"
    );

    let apply = common::run_ocl_cli(&["cassette", "prune", &artifact_s, "--apply", "--json"]);
    let apply_stdout = common::assert_success(&apply);
    let apply_json: JsonValue = serde_json::from_str(&apply_stdout).expect("parse prune apply");
    let applied_blocks = apply_json
        .get("applied_blocks")
        .and_then(JsonValue::as_u64)
        .unwrap_or(0);
    assert!(applied_blocks >= 1, "expected at least one applied block");
    assert!(
        !orphan_path.exists(),
        "orphan block must be deleted by prune --apply"
    );

    common::merge_cassette_operability_section(
        "prune_plan_apply",
        json!({
            "status": "PASS",
            "artifact_dir": artifact.to_string_lossy().replace('\\', "/"),
            "orphan_blocks_detected": orphan_blocks.len(),
            "applied_blocks": applied_blocks
        }),
    );
}
