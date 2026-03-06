use serde_json::json;

#[path = "v20_gate_f_common.rs"]
mod v20f;

#[test]
fn v20_secret_exfiltration_guard() {
    v20f::ensure_run_manifest();

    let samples = vec![
        "Authorization: Bearer sk_live_example_token",
        "api_key=abc123",
        "password=super_secret",
        "normal event without secret payload",
    ];

    let mut redaction_count = 0u64;
    let mut leak_count = 0u64;
    let mut events = Vec::<serde_json::Value>::new();

    for sample in samples {
        let redacted = v20f::redact_sensitive(sample);
        let had_marker = v20f::contains_secret_marker(sample);
        let still_sensitive = v20f::contains_secret_marker(&redacted);
        if had_marker {
            redaction_count += 1;
            if still_sensitive {
                leak_count += 1;
            }
        }
        events.push(json!({
            "input_had_sensitive_marker": had_marker,
            "output_has_sensitive_marker": still_sensitive,
            "output_preview": redacted
        }));
    }

    assert!(
        redaction_count > 0,
        "expected sensitive samples for guard validation"
    );
    assert_eq!(leak_count, 0, "secret marker leak detected after redaction");

    let report = json!({
        "schema": "ocl.w20.security.secrets_hygiene_report.v1",
        "status": "PASS",
        "samples_checked": events.len(),
        "redaction_count": redaction_count,
        "leak_count": leak_count,
        "events": events,
        "run_manifest_ref": "target/ocl/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20f::run_manifest_sha256()
    });
    v20f::write_report("security/secrets_hygiene_report.json", &report);
}
