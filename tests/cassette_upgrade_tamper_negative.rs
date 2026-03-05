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
    fs::write(cassette_dir.join("cassette_index.json"), "{\"call_id_to_entry_id\":{}}")
        .expect("write cassette_index.json");
    fs::write(cassette_dir.join("cassette_meta.toml"), "mode = \"record\"\n")
        .expect("write cassette_meta.toml");

    let artifact_s = artifact.to_string_lossy().to_string();
    let out = common::run_ocl_cli(&[
        "cassette",
        "upgrade",
        &artifact_s,
        "--apply",
        "--json",
    ]);
    common::assert_success(&out);
}

#[test]
fn cassette_upgrade_plan_detects_tampered_block_hash() {
    let root = common::temp_dir("cassette_upgrade_tamper");
    let artifact = common::ensure_artifact_layout(&root);
    seed_and_upgrade(&artifact);

    let index_json: JsonValue = serde_json::from_str(
        &fs::read_to_string(artifact.join("cassette").join("cassette_blocks_index.json"))
            .expect("read index"),
    )
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
    let mut tampered = fs::read_to_string(&block_path).expect("read block");
    tampered.push_str("{\"tamper\":true}");
    fs::write(&block_path, tampered).expect("write tampered block");

    let artifact_s = artifact.to_string_lossy().to_string();
    let out = common::run_ocl_cli(&["cassette", "upgrade", &artifact_s, "--plan"]);
    common::assert_failed_with(&out, "X-CASSETTE-UPGRADE-TAMPER");
}
