use std::fs;

#[path = "w17_gate_b_cli_common.rs"]
mod common;

#[test]
fn cassette_upgrade_respects_max_block_bytes_cap() {
    let root = common::temp_dir("dos_caps");
    let artifact = common::ensure_artifact_layout(&root);
    let cassette_dir = artifact.join("cassette");
    fs::create_dir_all(&cassette_dir).expect("create cassette dir");

    let oversized_payload = "x".repeat(1024);
    let line = format!(
        "{{\"id\":\"big-1\",\"type\":\"wallclock\",\"payload\":\"{}\"}}\n",
        oversized_payload
    );
    fs::write(cassette_dir.join("cassette.jsonl"), line).expect("write cassette jsonl");
    fs::write(cassette_dir.join("cassette_index.json"), "{\"call_id_to_entry_id\":{}}")
        .expect("write cassette index");
    fs::write(cassette_dir.join("cassette_meta.toml"), "mode = \"record\"\n")
        .expect("write cassette meta");

    let artifact_s = artifact.to_string_lossy().to_string();
    let out = common::run_ocl_cli(&[
        "cassette",
        "upgrade",
        &artifact_s,
        "--plan",
        "--max-block-bytes",
        "64",
    ]);
    common::assert_failed_with(&out, "X-CASSETTE-BLOCK-TOO-LARGE");
}
