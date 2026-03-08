use std::fs;

use serde_json::Value as JsonValue;

#[path = "w17_gate_b_cli_common.rs"]
mod common;

fn seed_v08_cassette(artifact: &std::path::Path) {
    let cassette_dir = artifact.join("cassette");
    fs::create_dir_all(&cassette_dir).expect("create cassette");
    let jsonl = concat!(
        "{\"id\":\"e1\",\"type\":\"wallclock\",\"unix_ms\":1}\n",
        "{\"id\":\"e1\",\"type\":\"wallclock\",\"unix_ms\":1}\n",
        "{\"id\":\"e2\",\"type\":\"wallclock\",\"unix_ms\":2}\n"
    );
    fs::write(cassette_dir.join("cassette.jsonl"), jsonl).expect("write cassette.jsonl");
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
}

#[test]
fn cassette_upgrade_v08_to_v17_creates_block_store_layout() {
    let root = common::temp_dir("upgrade_v08_to_v17");
    let artifact = common::ensure_artifact_layout(&root);
    seed_v08_cassette(&artifact);

    let artifact_s = artifact.to_string_lossy().to_string();
    let out = common::run_ocp_cli(&["cassette", "upgrade", &artifact_s, "--apply", "--json"]);
    let stdout = common::assert_success(&out);
    let parsed: JsonValue = serde_json::from_str(&stdout).expect("json output");

    let entries = parsed
        .get("entries")
        .and_then(JsonValue::as_u64)
        .unwrap_or(0);
    let blocks = parsed
        .get("unique_blocks")
        .and_then(JsonValue::as_u64)
        .unwrap_or(0);
    assert!(entries >= 3, "expected 3 logical entries");
    assert!(
        blocks >= 2 && blocks < entries,
        "expected deduped blocks (entries={entries}, blocks={blocks})"
    );

    assert!(
        artifact
            .join("cassette")
            .join("cassette_blocks_index.json")
            .exists(),
        "missing cassette_blocks_index.json"
    );
    assert!(
        artifact
            .join("cassette")
            .join("cassette_storage_meta.toml")
            .exists(),
        "missing cassette_storage_meta.toml"
    );
    assert!(
        artifact.join("cassette").join("blocks").is_dir(),
        "missing blocks directory"
    );
    assert!(
        root.join("target")
            .join("ocp")
            .join("w17")
            .join("cassette")
            .join("cassette_migration_report.json")
            .exists(),
        "missing cassette_migration_report.json"
    );
}
