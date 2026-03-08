use serde_json::Value as JsonValue;

#[path = "v18_gate_f_common.rs"]
mod common;

#[test]
fn v18_release_artifact_manifest_contains_required_artifacts_and_hashes() {
    let (manifest_path, sig_path) = common::ensure_release_artifact_manifest_signed();
    assert!(manifest_path.exists(), "missing release artifact manifest");
    assert!(
        sig_path.exists(),
        "missing canonical release manifest signature"
    );
    assert!(
        common::w18_rc_dir()
            .join("release_artifact_manifest.sig")
            .exists(),
        "missing compatibility signature alias release_artifact_manifest.sig"
    );

    let manifest = common::read_json(&manifest_path);
    assert_eq!(
        manifest
            .get("schema")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "ocp.w18.rc.release_artifact_manifest.v1"
    );
    let artifacts = manifest
        .get("artifacts")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(
        !artifacts.is_empty(),
        "release artifact manifest must include artifact rows"
    );

    let root = common::repo_root();
    for row in &artifacts {
        let rel = row
            .get("path")
            .and_then(JsonValue::as_str)
            .unwrap_or_else(|| panic!("artifact row missing `path`: {row}"));
        let expected = row
            .get("sha256")
            .and_then(JsonValue::as_str)
            .unwrap_or_else(|| panic!("artifact row missing `sha256`: {row}"));
        let actual = common::sha256_hex_file(&root.join(rel));
        assert_eq!(actual, expected, "artifact hash mismatch for `{rel}`");
    }
}
