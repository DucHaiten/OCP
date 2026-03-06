use serde_json::Value as JsonValue;

#[path = "v20_gate_g_common.rs"]
mod v20g;

#[test]
fn v20_release_package_manifest() {
    v20g::ensure_run_manifest();
    let fixture = v20g::ensure_release_fixture_v20();

    assert!(
        fixture.manifest_path.exists(),
        "missing v1_rc_manifest.json"
    );
    assert!(
        fixture.manifest_sig_path.exists(),
        "missing v1_rc_manifest.json.sig"
    );

    let manifest = v20g::read_json(&fixture.manifest_path);
    assert_eq!(
        manifest
            .get("schema")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "ocl.w20.release.v1_rc_manifest.v1"
    );
    assert_eq!(
        manifest
            .get("target_release")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "v1.0-rc"
    );

    let artifacts = manifest
        .get("artifacts")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(
        !artifacts.is_empty(),
        "release manifest artifacts must not be empty"
    );

    let root = v20g::repo_root();
    for row in &artifacts {
        let rel = row
            .get("path")
            .and_then(JsonValue::as_str)
            .unwrap_or_else(|| panic!("manifest row missing `path`: {row}"));
        let expected = row
            .get("sha256")
            .and_then(JsonValue::as_str)
            .unwrap_or_else(|| panic!("manifest row missing `sha256`: {row}"));
        let actual = v20g::sha256_hex_file(&root.join(rel));
        assert_eq!(actual, expected, "manifest hash mismatch for `{rel}`");
    }
}
