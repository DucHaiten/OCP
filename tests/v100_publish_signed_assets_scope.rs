#[path = "v100_gate_h_common.rs"]
mod v100h;

#[test]
fn v100_publish_signed_assets_scope() {
    let report = v100h::ensure_publish_signed_assets_scope_report();
    assert_eq!(
        report
            .get("status")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(""),
        "PASS"
    );
    let missing_disallowed = report
        .get("missing_disallowed")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(
        missing_disallowed.is_empty(),
        "signed-assets scope still has disallowed missing assets"
    );
    let doc_checks = report
        .get("docs_scope_checks")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(
        doc_checks.iter().all(|item| item
            .get("ok")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)),
        "verify-download companion docs must mention precedence + source trust scope"
    );
}
