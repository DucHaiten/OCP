use serde_json::json;

#[path = "v18_gate_d_common.rs"]
mod common;

#[test]
fn v18_privacy_no_secrets_gate_detects_sensitive_markers() {
    common::ensure_run_manifest();
    let root = common::temp_dir("privacy_no_secrets");
    let artifact = common::ensure_artifact_layout(&root);
    common::seed_v08_cassette_lines(
        &artifact,
        &["{\"id\":\"s1\",\"headers\":\"Authorization: Bearer secret-token\"}"],
    );

    let artifact_s = artifact.to_string_lossy().to_string();
    let out = common::run_ocl_cli(&["cassette", "stats", &artifact_s]);
    common::assert_failed_with(&out, "X-CASSETTE-PRIVACY-LEAK");

    let report = json!({
        "schema": "ocl.w18.security.privacy_hygiene_report.v1",
        "run_manifest_ref": "target/ocl/w18/meta/run_manifest.json",
        "artifact_dir": artifact.to_string_lossy().replace('\\', "/"),
        "reason_code": "X-CASSETTE-PRIVACY-LEAK",
        "status": "PASS"
    });
    common::write_json_pretty(
        &common::w18_security_dir().join("privacy_hygiene_report.json"),
        &report,
    );
}
