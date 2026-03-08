use ocp::ocp::{parse_program, DiagPhase, Diagnostic, ErrorCode, ReasonCode, Span};

#[test]
fn error_codes_have_stable_wire_values() {
    assert_eq!(ErrorCode::PUnexpectedToken.as_str(), "P-UNEXPECTED-TOKEN");
    assert_eq!(ErrorCode::TTypeMismatch.as_str(), "T-TYPE-MISMATCH");
    assert_eq!(ErrorCode::TImportNotFound.as_str(), "T-IMPORT-NOT-FOUND");
    assert_eq!(ErrorCode::TImportCycle.as_str(), "T-IMPORT-CYCLE");
    assert_eq!(ErrorCode::TForNotList.as_str(), "T-FOR-NOT-LIST");
    assert_eq!(ErrorCode::TCapNotIntLit.as_str(), "T-CAP-NOT-INT-LIT");
    assert_eq!(ErrorCode::TTryNotResult4.as_str(), "T-TRY-NOT-RESULT4");
    assert_eq!(ErrorCode::TGuardNotResult4.as_str(), "T-GUARD-NOT-RESULT4");
    assert_eq!(ErrorCode::TTryElseNoValue.as_str(), "T-TRY-ELSE-NO-VALUE");
    assert_eq!(ErrorCode::TCtxMissingField.as_str(), "T-CTX-MISSING-FIELD");
    assert_eq!(ErrorCode::TCtxUnknownField.as_str(), "T-CTX-UNKNOWN-FIELD");
    assert_eq!(ErrorCode::TCtxTypeMismatch.as_str(), "T-CTX-TYPE-MISMATCH");
    assert_eq!(
        ErrorCode::TCtxConstraintViolation.as_str(),
        "T-CTX-CONSTRAINT-VIOLATION"
    );
    assert_eq!(
        ErrorCode::TKeyNotLiteralForSchema.as_str(),
        "T-KEY-NOT-LITERAL-FOR-SCHEMA"
    );
    assert_eq!(ErrorCode::XCommitForbidden.as_str(), "X-COMMIT-FORBIDDEN");
    assert_eq!(ErrorCode::XLoopCapExceeded.as_str(), "X-LOOP-CAP-EXCEEDED");
    assert_eq!(ErrorCode::XKeysCapExceeded.as_str(), "X-KEYS-CAP-EXCEEDED");
    assert_eq!(ErrorCode::XGuardFailed.as_str(), "X-GUARD-FAILED");
    assert_eq!(ErrorCode::RCapabilityDenied.as_str(), "R-CAPABILITY-DENIED");
    assert_eq!(ErrorCode::RCtxInvalid.as_str(), "R-CTX-INVALID");
    assert_eq!(ErrorCode::XConditionFalse.as_str(), "X-COND-FALSE");
    assert_eq!(ErrorCode::XConditionDeferred.as_str(), "X-COND-DEFERRED");
    assert_eq!(
        ErrorCode::XConditionInsufficient.as_str(),
        "X-COND-INSUFFICIENT"
    );
}

#[test]
fn e_aliases_map_to_canonical_taxonomy() {
    assert_eq!(ErrorCode::ECommitForbidden.as_str(), "X-COMMIT-FORBIDDEN");
    assert_eq!(ErrorCode::EConditionFalse.as_str(), "X-COND-FALSE");
    assert_eq!(
        ErrorCode::XCommitForbidden.legacy_alias(),
        Some("E-COMMIT-FORBIDDEN")
    );
}

#[test]
fn reason_codes_match_v0_1_baseline() {
    assert_eq!(ReasonCode::BudgetExceeded.as_str(), "RC-BUDGET-EXCEEDED");
    assert_eq!(ReasonCode::CtxInvalid.as_str(), "RC-CTX-INVALID");
    assert_eq!(ReasonCode::JsonInvalid.as_str(), "RC-JSON-INVALID");
    assert_eq!(ReasonCode::NotImplemented.as_str(), "RC-NOT-IMPLEMENTED");
}

#[test]
fn diagnostic_keeps_code_span_message_and_hint() {
    let span = Span::new(2, 10, 15, 1, 11);
    let d = Diagnostic::new(
        ErrorCode::PInvalidLiteral,
        DiagPhase::Parse,
        span,
        "invalid integer literal",
    )
    .with_hint("use base-10 digits only");

    assert_eq!(d.code, ErrorCode::PInvalidLiteral);
    assert_eq!(d.phase, DiagPhase::Parse);
    assert_eq!(d.span, span);
    assert_eq!(d.message, "invalid integer literal");
    assert_eq!(d.hint.as_deref(), Some("use base-10 digits only"));
    assert_eq!(d.root_reason, None);
}

#[test]
fn parse_error_has_code_span_and_hint() {
    let src = "let = 1;";
    let err = parse_program(src, 7).expect_err("parse should fail");
    assert!(err.code.as_str().starts_with("P-"));
    assert_eq!(err.phase, DiagPhase::Parse);
    assert_eq!(err.span.file_id, 7);
    assert!(err.span.end >= err.span.start);
    assert!(err.hint.is_some());
}
