use std::fs;

use serde_json::json;

#[path = "v19_gate_f_common.rs"]
mod f19;

#[test]
fn v19_vsix_package_builds_release_manifest_fixture() {
    f19::ensure_run_manifest();
    let fixture = f19::ensure_release_fixture_v19();

    assert!(fixture.vsix_path.exists(), "vsix fixture must exist");
    assert!(
        fixture.manifest_path.exists(),
        "release manifest must exist"
    );
    assert!(
        fixture.manifest_sig_path.exists(),
        "release manifest signature must exist"
    );
    for path in fixture.binaries.values() {
        assert!(path.exists(), "bundled binary missing: {}", path.display());
    }

    let manifest = f19::read_json(&fixture.manifest_path);
    let manifest_vsix_hash = manifest
        .get("vsix_sha256")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    assert_eq!(manifest_vsix_hash, fixture.vsix_sha256);

    let report = json!({
        "schema": "ocl.w19.release.editor_release_manifest.v1",
        "status": "PASS",
        "manifest_path": fixture.manifest_path.to_string_lossy().replace('\\', "/"),
        "manifest_sig_path": fixture.manifest_sig_path.to_string_lossy().replace('\\', "/"),
        "vsix_path": fixture.vsix_path.to_string_lossy().replace('\\', "/"),
        "vsix_size_bytes": fs::metadata(&fixture.vsix_path).map(|m| m.len()).unwrap_or(0),
        "run_manifest_ref": "target/ocl/w19/meta/run_manifest.json",
        "run_manifest_sha256": f19::run_manifest_sha256()
    });
    f19::write_report("release/editor_release_manifest.json", &manifest);
    f19::write_report("release/vsix_package_report.json", &report);
}
