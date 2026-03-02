use ocp_ocl::ocp_ocl::{DiagPhase, Diagnostic, ErrorCode, ReasonCode, Span};

#[test]
fn error_codes_have_stable_wire_values() {
    assert_eq!(ErrorCode::PUnexpectedToken.as_str(), "P-UNEXPECTED-TOKEN");
    assert_eq!(ErrorCode::TTypeMismatch.as_str(), "T-TYPE-MISMATCH");
    assert_eq!(ErrorCode::XCommitForbidden.as_str(), "X-COMMIT-FORBIDDEN");
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
