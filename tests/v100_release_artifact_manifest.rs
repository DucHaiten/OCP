use serde_json::Value as JsonValue;

#[path = "v100_gate_b_common.rs"]
mod v100b;

#[test]
fn v100_release_artifact_manifest() {
    let fixture = v100b::ensure_release_fixture_v100();
    assert!(
        fixture.manifest_path.exists(),
        "missing release_artifact_manifest.json"
    );

    let manifest = v100b::read_manifest();
    assert_eq!(
        manifest
            .get("schema")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "ocl.release.artifact_manifest.v1"
    );
    assert_eq!(
        manifest
            .get("release_version")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "v1.0.0"
    );
    assert_eq!(
        manifest
            .get("release_channel")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "github_release"
    );
    assert_eq!(
        manifest
            .get("toolchain_digest_ref")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "target/ocl/w100/meta/run_manifest.json"
    );

    let artifacts = manifest
        .get("artifacts")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(!artifacts.is_empty(), "manifest artifacts must not be empty");

    for row in &artifacts {
        let rel = row
            .get("path")
            .and_then(JsonValue::as_str)
            .unwrap_or_else(|| panic!("manifest row missing path: {row}"));
        let expected = row
            .get("sha256")
            .and_then(JsonValue::as_str)
            .unwrap_or_else(|| panic!("manifest row missing sha256: {row}"));
        let signature_status = row
            .get("signature_status")
            .and_then(JsonValue::as_str)
            .unwrap_or_default();
        assert!(
            signature_status == "signed" || signature_status == "unsigned_or_external",
            "invalid signature_status for `{rel}`"
        );
        let actual = v100b::sha256_hex_file(&v100b::repo_root().join(rel));
        assert_eq!(actual, expected, "manifest hash mismatch for `{rel}`");
    }
}
