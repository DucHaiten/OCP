use std::fs;

use serde_json::{json, Value as JsonValue};

#[path = "v18_gate_d_common.rs"]
mod common;

#[test]
fn v18_dos_caps_gate_enforces_entry_caps_and_missing_chunk_fail_honest() {
    common::ensure_run_manifest();

    let cap_root = common::temp_dir("dos_caps_entry_limit");
    let cap_artifact = common::ensure_artifact_layout(&cap_root);
    common::seed_v08_cassette_lines(
        &cap_artifact,
        &[
            "{\"id\":\"e1\",\"type\":\"wallclock\",\"unix_ms\":1}",
            "{\"id\":\"e2\",\"type\":\"wallclock\",\"unix_ms\":2}",
        ],
    );
    let cap_artifact_s = cap_artifact.to_string_lossy().to_string();
    let capped = common::run_ocl_cli(&[
        "cassette",
        "upgrade",
        &cap_artifact_s,
        "--plan",
        "--max-entries",
        "1",
    ]);
    common::assert_failed_with(&capped, "X-CASSETTE-MAX-ENTRIES");

    let missing_root = common::temp_dir("dos_caps_missing_chunk");
    let missing_artifact = common::ensure_artifact_layout(&missing_root);
    common::seed_v08_cassette_lines(
        &missing_artifact,
        &["{\"id\":\"z1\",\"type\":\"wallclock\",\"unix_ms\":9}"],
    );
    let missing_artifact_s = missing_artifact.to_string_lossy().to_string();
    let upgraded = common::run_ocl_cli(&[
        "cassette",
        "upgrade",
        &missing_artifact_s,
        "--apply",
        "--json",
    ]);
    let upgraded_stdout = common::assert_success(&upgraded);
    let upgraded_json: JsonValue = serde_json::from_str(&upgraded_stdout).expect("upgrade json");
    let index_path = upgraded_json
        .get("index_path")
        .and_then(JsonValue::as_str)
        .expect("index_path")
        .to_string();
    let index_json: JsonValue =
        serde_json::from_str(&fs::read_to_string(&index_path).expect("read cassette blocks index"))
            .expect("parse cassette blocks index");
    let first_block = index_json
        .get("block_ids")
        .and_then(JsonValue::as_array)
        .and_then(|arr| arr.first())
        .and_then(JsonValue::as_str)
        .expect("first block id");
    let block_path = missing_artifact
        .join("cassette")
        .join("blocks")
        .join(format!("{first_block}.json"));
    fs::remove_file(&block_path).expect("remove referenced block");

    let gc = common::run_ocl_cli(&["cassette", "gc", &missing_artifact_s]);
    common::assert_failed_with(&gc, "X-CASSETTE-MISSING-BLOCK");

    let report = json!({
        "schema": "ocl.w18.security.dos_caps_report.v1",
        "run_manifest_ref": "target/ocl/w18/meta/run_manifest.json",
        "entry_cap_check": {
            "artifact_dir": cap_artifact.to_string_lossy().replace('\\', "/"),
            "reason_code": "X-CASSETTE-MAX-ENTRIES",
            "status": "PASS"
        },
        "missing_chunk_fail_honest_check": {
            "artifact_dir": missing_artifact.to_string_lossy().replace('\\', "/"),
            "reason_code": "X-CASSETTE-MISSING-BLOCK",
            "status": "PASS"
        },
        "status": "PASS"
    });
    common::write_json_pretty(
        &common::w18_security_dir().join("dos_caps_report.json"),
        &report,
    );
}
