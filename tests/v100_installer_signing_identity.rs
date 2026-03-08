use serde_json::{json, Value as JsonValue};

#[path = "v100_gate_c_common.rs"]
mod v100c;

#[test]
fn v100_installer_signing_identity() {
    let contract = v100c::installer_toolchain_contract();
    let signing_mode = contract
        .get("code_signing_mode")
        .and_then(JsonValue::as_str)
        .unwrap_or("");
    let subject = contract
        .get("authenticode_expected_subject")
        .and_then(JsonValue::as_str)
        .unwrap_or("");
    let thumbprint = contract
        .get("authenticode_thumbprint")
        .and_then(JsonValue::as_str)
        .unwrap_or("");

    if signing_mode == "none" {
        assert!(
            subject.trim().is_empty(),
            "authenticode_expected_subject must be empty when signing mode is none"
        );
        assert!(
            thumbprint.trim().is_empty(),
            "authenticode_thumbprint must be empty when signing mode is none"
        );
    } else if signing_mode == "authenticode" {
        assert!(
            !subject.trim().is_empty() || !thumbprint.trim().is_empty(),
            "authenticode mode requires expected subject or thumbprint"
        );
    } else {
        panic!("unsupported code_signing_mode `{signing_mode}`");
    }

    let report = json!({
        "schema": "ocp.w100.install.installer_signing_identity_report.v1",
        "status": "PASS",
        "code_signing_mode": signing_mode,
        "authenticode_expected_subject": subject,
        "authenticode_thumbprint": thumbprint,
        "run_manifest_ref": "target/ocp/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100c::run_manifest_sha256()
    });
    v100c::write_report("install/installer_signing_identity_report.json", &report);
}
