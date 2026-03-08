use std::fs;

use serde_json::{json, Value as JsonValue};
use sha2::{Digest, Sha256};

#[path = "v18_gate_d_common.rs"]
mod common;

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[test]
fn v18_cassette_dedup_integrity_uses_content_addressed_blocks() {
    common::ensure_run_manifest();
    let root = common::temp_dir("dedup_integrity");
    let artifact = common::ensure_artifact_layout(&root);
    common::seed_v08_cassette_lines(
        &artifact,
        &[
            "{\"id\":\"row-1\",\"type\":\"wallclock\",\"unix_ms\":100}",
            "{\"id\":\"row-1\",\"type\":\"wallclock\",\"unix_ms\":100}",
            "{\"id\":\"row-2\",\"type\":\"wallclock\",\"unix_ms\":200}",
        ],
    );

    let artifact_s = artifact.to_string_lossy().to_string();
    let out = common::run_ocp_cli(&["cassette", "upgrade", &artifact_s, "--apply", "--json"]);
    let stdout = common::assert_success(&out);
    let parsed: JsonValue = serde_json::from_str(&stdout).expect("parse upgrade json");
    let unique_blocks = parsed
        .get("unique_blocks")
        .and_then(JsonValue::as_u64)
        .unwrap_or(0);
    assert!(
        unique_blocks >= 2,
        "dedup integrity requires at least 2 unique blocks, got {unique_blocks}"
    );

    let blocks_dir = artifact.join("cassette").join("blocks");
    let mut verified = 0u64;
    for entry in fs::read_dir(&blocks_dir).expect("read blocks dir") {
        let path = entry.expect("dir entry").path();
        if !path.is_file() {
            continue;
        }
        let stem = path
            .file_stem()
            .expect("file stem")
            .to_string_lossy()
            .to_string();
        let bytes = fs::read(&path).expect("read block bytes");
        let actual = sha256_hex(&bytes);
        assert_eq!(
            actual, stem,
            "content-addressed block filename must match sha256 payload"
        );
        verified += 1;
    }
    assert_eq!(
        verified, unique_blocks,
        "block file count mismatch: verified={verified}, unique_blocks={unique_blocks}"
    );

    common::merge_cassette_operability_section(
        "dedup_integrity",
        json!({
            "status": "PASS",
            "artifact_dir": artifact.to_string_lossy().replace('\\', "/"),
            "unique_blocks": unique_blocks,
            "verified_blocks": verified
        }),
    );
}
