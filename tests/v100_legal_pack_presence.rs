use serde_json::{json, Value as JsonValue};

#[path = "v100_gate_f_common.rs"]
mod v100f;

#[test]
fn v100_legal_pack_presence() {
    v100f::ensure_run_manifest();

    let contract = v100f::legal_pack_required();
    assert_eq!(
        contract
            .get("contract_id")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "v1.legal_pack_required"
    );

    let mut root_verified = Vec::<String>::new();
    for rel in contract
        .get("required_root_files")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default()
        .iter()
        .filter_map(JsonValue::as_str)
    {
        let path = v100f::repo_root().join(rel);
        assert!(path.exists(), "missing required root legal file `{rel}`");
        root_verified.push(rel.to_string());
    }

    let mut docs_verified = Vec::<String>::new();
    for rel in contract
        .get("required_docs_files")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default()
        .iter()
        .filter_map(JsonValue::as_str)
    {
        let path = v100f::repo_root().join(rel);
        assert!(path.exists(), "missing required legal doc `{rel}`");
        docs_verified.push(rel.to_string());
    }

    let report = json!({
        "schema": "ocp.w100.legal.legal_pack_report.v1",
        "status": "PASS",
        "required_root_files_verified": root_verified,
        "required_docs_files_verified": docs_verified,
        "run_manifest_ref": "target/ocp/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100f::run_manifest_sha256()
    });
    v100f::write_report("legal/legal_pack_report.json", &report);
}
