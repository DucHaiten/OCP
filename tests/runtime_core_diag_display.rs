use ocp_runtime_core::{DiagPhase, Diagnostic, ErrorCode, ReasonCode, RuntimeCoreError, Span};

#[test]
fn runtime_core_diagnostic_display_includes_location_reason_and_hint() {
    let diag = Diagnostic::new(
        ErrorCode::RCapabilityDenied,
        DiagPhase::Exec,
        Span::new(9, 10, 20, 4, 7),
        "observe ctx must evaluate to map-compatible value",
    )
    .with_root_reason(ReasonCode::CtxInvalid)
    .with_hint("observe callsite bind `wr`");
    let rendered = RuntimeCoreError::Diagnostic(diag).to_string();
    assert!(
        rendered.contains("R-CAPABILITY-DENIED: observe ctx must evaluate to map-compatible value")
    );
    assert!(rendered.contains("module=file_id:9"), "actual={rendered}");
    assert!(rendered.contains("line=4"), "actual={rendered}");
    assert!(rendered.contains("column=7"), "actual={rendered}");
    assert!(
        rendered.contains("root_reason=RC-CTX-INVALID"),
        "actual={rendered}"
    );
    assert!(
        rendered.contains("hint: observe callsite bind `wr`"),
        "actual={rendered}"
    );
}
