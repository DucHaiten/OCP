use std::fs;

use serde_json::json;

#[path = "v18_gate_a_common.rs"]
mod v18;

#[test]
fn v18_canonical_serializer_contract() {
    v18::ensure_run_manifest();
    let path = v18::contracts_root()
        .join("serialization")
        .join("canonical_serializer.v1.json");
    let value = v18::read_json(&path);
    assert_eq!(
        value
            .get("contract_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(""),
        "v1.canonical_serializer"
    );
    assert_eq!(
        value
            .get("canonical_json")
            .and_then(serde_json::Value::as_object)
            .and_then(|map| map.get("key_order"))
            .and_then(serde_json::Value::as_str),
        Some("utf8-byte-lexicographic")
    );
    assert_eq!(
        value
            .get("canonical_text")
            .and_then(serde_json::Value::as_object)
            .and_then(|map| map.get("newline"))
            .and_then(serde_json::Value::as_str),
        Some("LF")
    );

    let report = json!({
        "schema": "ocp.w18.canonical_serializer_contract_report.v1",
        "run_manifest_ref": "target/ocp/w18/meta/run_manifest.json",
        "contract_path": "contracts/serialization/canonical_serializer.v1.json",
        "status": "PASS"
    });
    let out_dir = v18::w18_target_root().join("contracts");
    fs::create_dir_all(&out_dir).expect("create w18 contracts output dir");
    v18::write_json_pretty(
        &out_dir.join("canonical_serializer_contract_report.json"),
        &report,
    );
}
