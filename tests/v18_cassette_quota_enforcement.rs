use serde_json::json;

#[path = "v18_gate_d_common.rs"]
mod common;

#[test]
fn v18_cassette_quota_enforcement_is_fail_honest_and_reported() {
    common::ensure_run_manifest();
    let root = common::temp_dir("quota_enforcement");
    let artifact = common::ensure_artifact_layout(&root);

    let oversized_payload = "x".repeat(2048);
    let line = format!(
        "{{\"id\":\"quota-1\",\"type\":\"wallclock\",\"payload\":\"{}\"}}",
        oversized_payload
    );
    common::seed_v08_cassette_lines(&artifact, &[&line]);

    let artifact_s = artifact.to_string_lossy().to_string();
    let out = common::run_ocl_cli(&[
        "cassette",
        "upgrade",
        &artifact_s,
        "--plan",
        "--max-block-bytes",
        "64",
        "--json",
    ]);
    common::assert_failed_with(&out, "X-CASSETTE-BLOCK-TOO-LARGE");

    common::merge_cassette_operability_section(
        "quota_enforcement",
        json!({
            "status": "PASS",
            "artifact_dir": artifact.to_string_lossy().replace('\\', "/"),
            "reason_code": "X-CASSETTE-BLOCK-TOO-LARGE"
        }),
    );
}
