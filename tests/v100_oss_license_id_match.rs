use serde_json::{json, Value as JsonValue};

#[path = "v100_gate_g_common.rs"]
mod v100g;

#[test]
fn v100_oss_license_id_match() {
    v100g::ensure_run_manifest();

    let contract = v100g::licensing_model();
    let license_id = contract
        .get("oss_license_id")
        .and_then(JsonValue::as_str)
        .unwrap_or("");
    let license_file = contract
        .get("oss_license_file")
        .and_then(JsonValue::as_str)
        .unwrap_or("");

    assert_eq!(license_id, "AGPL-3.0-only");
    assert_eq!(license_file, "LICENSE");

    let license_path = v100g::repo_root().join(license_file);
    assert!(license_path.exists(), "LICENSE file declared by contract must exist");
    let license = v100g::read_text(&license_path);
    assert!(
        license.contains("GNU AFFERO GENERAL PUBLIC LICENSE")
            && license.contains("Version 3, 19 November 2007"),
        "root LICENSE must match AGPL-3.0-only"
    );

    let report = json!({
        "schema": "ocl.w100.business.oss_license_match_report.v1",
        "status": "PASS",
        "oss_license_id": "AGPL-3.0-only",
        "oss_license_file": "LICENSE",
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100g::run_manifest_sha256()
    });
    v100g::write_report("business/oss_license_match_report.json", &report);
}
