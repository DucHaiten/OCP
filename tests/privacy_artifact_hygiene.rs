use std::fs;

#[path = "w17_gate_b_cli_common.rs"]
mod common;

#[test]
fn cassette_stats_fails_when_sensitive_markers_are_present() {
    let root = common::temp_dir("privacy_hygiene");
    let artifact = common::ensure_artifact_layout(&root);
    let cassette_dir = artifact.join("cassette");
    fs::create_dir_all(&cassette_dir).expect("create cassette dir");
    fs::write(
        cassette_dir.join("cassette.jsonl"),
        "{\"id\":\"s1\",\"headers\":\"Authorization: Bearer secret-token\"}\n",
    )
    .expect("write cassette");

    let artifact_s = artifact.to_string_lossy().to_string();
    let out = common::run_ocp_cli(&["cassette", "stats", &artifact_s]);
    common::assert_failed_with(&out, "X-CASSETTE-PRIVACY-LEAK");
}
