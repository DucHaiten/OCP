use std::fs;

use serde_json::Value as JsonValue;

#[path = "w17_gate_b_cli_common.rs"]
mod common;

fn seed_and_upgrade(artifact: &std::path::Path) -> String {
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
    let stdout = common::assert_success(&out);
    let parsed: JsonValue = serde_json::from_str(&stdout).expect("upgrade json");
    parsed
        .get("index_path")
        .and_then(JsonValue::as_str)
        .unwrap_or_default()
        .to_string()
}

#[test]
fn cassette_gc_fails_when_referenced_block_is_missing() {
    let root = common::temp_dir("cassette_gc_negative");
    let artifact = common::ensure_artifact_layout(&root);
    let index_path_text = seed_and_upgrade(&artifact);
    let index_path = std::path::PathBuf::from(index_path_text);

    let index_json: JsonValue =
        serde_json::from_str(&fs::read_to_string(&index_path).expect("read index"))
            .expect("parse index json");
    let first_block = index_json
        .get("block_ids")
        .and_then(JsonValue::as_array)
        .and_then(|arr| arr.first())
        .and_then(JsonValue::as_str)
        .expect("first block id")
        .to_string();
    let block_path = artifact
        .join("cassette")
        .join("blocks")
        .join(format!("{first_block}.json"));
    fs::remove_file(&block_path).expect("remove referenced block");

    let artifact_s = artifact.to_string_lossy().to_string();
    let out = common::run_ocp_cli(&["cassette", "gc", &artifact_s]);
    common::assert_failed_with(&out, "X-CASSETTE-MISSING-BLOCK");
}
