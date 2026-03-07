use serde_json::{json, Value as JsonValue};

#[path = "v100_gate_b_common.rs"]
mod v100b;

#[test]
fn v100_release_sbom_status_policy() {
    let fixture = v100b::ensure_release_fixture_v100();
    let matrix = v100b::release_asset_matrix();
    let installer_sbom = matrix
        .get("installer_sbom")
        .and_then(JsonValue::as_object)
        .cloned()
        .expect("installer_sbom object");
    let status = installer_sbom
        .get("status")
        .and_then(JsonValue::as_str)
        .unwrap_or("");
    assert!(
        status == "required" || status == "optional" || status == "unavailable",
        "invalid installer_sbom.status"
    );

    let installer_path = fixture.release_root.join("SBOM-installer.spdx.json");
    let installer_exists = installer_path.exists();
    if status == "required" {
        assert!(installer_exists, "installer SBOM is required but missing");
    }
    if status == "unavailable" {
        let reason = installer_sbom
            .get("unavailable_reason_code")
            .and_then(JsonValue::as_str)
            .unwrap_or("");
        assert!(
            !reason.trim().is_empty(),
            "installer_sbom.status=unavailable requires unavailable_reason_code"
        );
    }

    let report = json!({
        "schema": "ocl.w100.release.release_sbom_status_report.v1",
        "status": "PASS",
        "installer_sbom_status": status,
        "installer_sbom_exists": installer_exists,
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100b::run_manifest_sha256()
    });
    v100b::write_report("release/release_sbom_status_report.json", &report);
}
