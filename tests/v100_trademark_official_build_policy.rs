use serde_json::{json, Value as JsonValue};

#[path = "v100_gate_f_common.rs"]
mod v100f;

#[test]
fn v100_trademark_official_build_policy() {
    v100f::ensure_run_manifest();

    let contract = v100f::trademark_policy();
    let definition = contract
        .get("official_build_definition")
        .and_then(JsonValue::as_object)
        .unwrap_or_else(|| panic!("trademark policy missing official_build_definition"));

    let signer_ref = definition
        .get("required_signer_ref")
        .and_then(JsonValue::as_str)
        .unwrap_or("");
    let manifest_ref = definition
        .get("required_release_manifest")
        .and_then(JsonValue::as_str)
        .unwrap_or("");
    let signature_ref = definition
        .get("required_release_signature")
        .and_then(JsonValue::as_str)
        .unwrap_or("");

    assert!(
        v100f::repo_root().join(signer_ref).exists(),
        "missing signer ref"
    );
    assert!(
        v100f::repo_root().join(manifest_ref).exists(),
        "missing release manifest ref"
    );
    assert!(
        v100f::repo_root().join(signature_ref).exists(),
        "missing release signature ref"
    );

    let root = v100f::read_text(&v100f::repo_root().join("TRADEMARK.md"));
    let vi = v100f::read_text(&v100f::repo_root().join("docs/vi/legal/trademark.md"));
    let en = v100f::read_text(&v100f::repo_root().join("docs/en/legal/trademark.md"));

    for (label, content) in [("root", &root), ("vi", &vi), ("en", &en)] {
        assert!(
            content.contains("trust chain") || content.contains("trust chain chính thức"),
            "{label} trademark text must mention trust chain"
        );
        assert!(
            content.contains("Official OCP Build") || content.contains("bản chính thức"),
            "{label} trademark text must define official build"
        );
    }

    let report = json!({
        "schema": "ocp.w100.legal.trademark_official_build_policy_report.v1",
        "status": "PASS",
        "required_signer_ref": signer_ref,
        "required_release_manifest": manifest_ref,
        "required_release_signature": signature_ref,
        "official_build_definition_verified": true,
        "run_manifest_ref": "target/ocp/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100f::run_manifest_sha256()
    });
    v100f::write_report("legal/trademark_official_build_policy_report.json", &report);
}
