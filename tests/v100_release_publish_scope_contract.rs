use serde_json::{json, Value as JsonValue};

#[path = "v100_gate_b_common.rs"]
mod v100b;

#[test]
fn v100_release_publish_scope_contract() {
    let fixture = v100b::ensure_release_fixture_v100();
    let scope = v100b::release_publish_scope();
    assert_eq!(
        scope
            .get("contract_id")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "v1.release_publish_scope"
    );

    let signed_assets_scope = scope
        .get("signed_assets_scope")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect::<Vec<String>>();
    assert!(
        signed_assets_scope
            .iter()
            .any(|item| item == "release_artifact_manifest.json"),
        "signed_assets_scope must include release_artifact_manifest.json"
    );
    assert!(
        signed_assets_scope
            .iter()
            .any(|item| item == "release_artifact_manifest.sig"),
        "signed_assets_scope must include release_artifact_manifest.sig"
    );

    let source_scope = scope
        .get("source_trust_scope")
        .and_then(JsonValue::as_object)
        .cloned()
        .expect("source_trust_scope object");
    let github_source_trusted = source_scope
        .get("github_auto_generated_source_archives_trusted")
        .and_then(JsonValue::as_bool)
        .unwrap_or(true);
    assert!(
        !github_source_trusted,
        "github auto-generated source archives must be out of trust chain by default"
    );

    let report = json!({
        "schema": "ocl.w100.release.release_publish_scope_report.v1",
        "status": "PASS",
        "signed_assets_scope": signed_assets_scope,
        "source_trust_scope": source_scope,
        "release_manifest_ref": fixture.manifest_path.to_string_lossy().replace('\\', "/"),
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100b::run_manifest_sha256()
    });
    v100b::write_report("release/release_publish_scope_report.json", &report);
}
