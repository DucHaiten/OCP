use serde_json::json;

#[path = "v100_gate_b_common.rs"]
mod v100b;

#[test]
fn v100_release_artifact_signature() {
    let fixture = v100b::ensure_release_fixture_v100();
    assert!(
        fixture.manifest_sig_path.exists(),
        "missing release_artifact_manifest.json.sig"
    );
    assert!(
        fixture.checksums_sig_path.exists(),
        "missing SHA256SUMS.sig"
    );

    let (manifest_ok, manifest_field) =
        v100b::verify_file_signature(&fixture.manifest_path, &fixture.manifest_sig_path);
    assert!(
        manifest_ok,
        "manifest signature verify failed at field `{manifest_field}`"
    );
    let (checksums_ok, checksums_field) =
        v100b::verify_file_signature(&fixture.checksums_path, &fixture.checksums_sig_path);
    assert!(
        checksums_ok,
        "checksums signature verify failed at field `{checksums_field}`"
    );

    let report = json!({
        "schema": "ocp.w100.release.release_artifact_signature_report.v1",
        "status": "PASS",
        "manifest_signature_verified": manifest_ok,
        "checksums_signature_verified": checksums_ok,
        "run_manifest_ref": "target/ocp/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100b::run_manifest_sha256()
    });
    v100b::write_report("release/release_artifact_signature_report.json", &report);
}
