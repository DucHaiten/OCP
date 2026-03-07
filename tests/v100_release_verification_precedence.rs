use serde_json::{json, Value as JsonValue};

#[path = "v100_gate_b_common.rs"]
mod v100b;

#[test]
fn v100_release_verification_precedence() {
    v100b::ensure_release_fixture_v100();
    let scope = v100b::release_publish_scope();
    let precedence = scope
        .get("verification_precedence")
        .and_then(JsonValue::as_object)
        .cloned()
        .expect("verification_precedence object");

    let primary = precedence
        .get("primary")
        .and_then(JsonValue::as_str)
        .unwrap_or("");
    let secondary = precedence
        .get("secondary")
        .and_then(JsonValue::as_str)
        .unwrap_or("");
    let on_mismatch = precedence
        .get("on_mismatch")
        .and_then(JsonValue::as_str)
        .unwrap_or("");

    assert_eq!(
        primary, "release_artifact_manifest.json.sig",
        "primary verification source must be release manifest signature"
    );
    assert_eq!(
        secondary, "SHA256SUMS.sig",
        "secondary verification source must be checksums signature"
    );
    assert_eq!(
        on_mismatch, "fail_no_fallback",
        "verification mismatch must be fail_no_fallback"
    );

    let report = json!({
        "schema": "ocl.w100.release.release_verification_precedence_report.v1",
        "status": "PASS",
        "primary": primary,
        "secondary": secondary,
        "on_mismatch": on_mismatch,
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100b::run_manifest_sha256()
    });
    v100b::write_report("release/release_verification_precedence_report.json", &report);
}
