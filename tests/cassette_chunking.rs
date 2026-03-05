use std::fs;

use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};

#[path = "w17_gate_b_cli_common.rs"]
mod common;

fn seed_v08_cassette(artifact: &std::path::Path) {
    let cassette_dir = artifact.join("cassette");
    fs::create_dir_all(&cassette_dir).expect("create cassette");
    let jsonl = concat!(
        "{\"id\":\"row-1\",\"type\":\"wallclock\",\"unix_ms\":100}\n",
        "{\"id\":\"row-1\",\"type\":\"wallclock\",\"unix_ms\":100}\n",
        "{\"id\":\"row-2\",\"type\":\"wallclock\",\"unix_ms\":200}\n"
    );
    fs::write(cassette_dir.join("cassette.jsonl"), jsonl).expect("write cassette.jsonl");
    fs::write(cassette_dir.join("cassette_index.json"), "{\"call_id_to_entry_id\":{}}")
        .expect("write cassette_index.json");
    fs::write(cassette_dir.join("cassette_meta.toml"), "mode = \"record\"\n")
        .expect("write cassette_meta.toml");
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[test]
fn cassette_chunking_writes_content_addressed_blocks() {
    let root = common::temp_dir("cassette_chunking");
    let artifact = common::ensure_artifact_layout(&root);
    seed_v08_cassette(&artifact);

    let artifact_s = artifact.to_string_lossy().to_string();
    let out = common::run_ocl_cli(&[
        "cassette",
        "upgrade",
        &artifact_s,
        "--apply",
        "--json",
    ]);
    let stdout = common::assert_success(&out);
    let parsed: JsonValue = serde_json::from_str(&stdout).expect("json output");
    let block_count = parsed
        .get("unique_blocks")
        .and_then(JsonValue::as_u64)
        .unwrap_or(0) as usize;

    let blocks_dir = artifact.join("cassette").join("blocks");
    let mut seen = 0usize;
    for entry in fs::read_dir(&blocks_dir).expect("read blocks dir") {
        let path = entry.expect("entry").path();
        if !path.is_file() {
            continue;
        }
        let stem = path
            .file_stem()
            .expect("stem")
            .to_string_lossy()
            .to_string();
        let bytes = fs::read(&path).expect("read block bytes");
        let actual = sha256_hex(&bytes);
        assert_eq!(actual, stem, "block filename must match sha256 content hash");
        seen += 1;
    }
    assert_eq!(seen, block_count, "block file count mismatch");
}
