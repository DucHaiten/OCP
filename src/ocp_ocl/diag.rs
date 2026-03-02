use crate::ocp_ocl::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagPhase {
    Parse,
    Typecheck,
    Exec,
    Runtime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    PUnexpectedToken,
    PUnexpectedEof,
    PInvalidLiteral,
    TUnknownIdentifier,
    TTypeMismatch,
    TMatchArmsIncomplete,
    EConditionFalse,
    EBudgetExceeded,
    ECommitForbidden,
    ECapabilityDenied,
}

impl ErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PUnexpectedToken => "P-UNEXPECTED-TOKEN",
            Self::PUnexpectedEof => "P-UNEXPECTED-EOF",
            Self::PInvalidLiteral => "P-INVALID-LITERAL",
            Self::TUnknownIdentifier => "T-UNKNOWN-IDENTIFIER",
            Self::TTypeMismatch => "T-TYPE-MISMATCH",
            Self::TMatchArmsIncomplete => "T-MATCH-ARMS-INCOMPLETE",
            Self::EConditionFalse => "E-CONDITION-FALSE",
            Self::EBudgetExceeded => "E-BUDGET-EXCEEDED",
            Self::ECommitForbidden => "E-COMMIT-FORBIDDEN",
            Self::ECapabilityDenied => "E-CAPABILITY-DENIED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReasonCode {
    BudgetExceeded,
    CapabilityDenied,
    KeyUnknown,
    CtxInvalid,
    PolicyDenied,
    AdapterFailed,
    NotImplemented,
}

impl ReasonCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BudgetExceeded => "RC-BUDGET-EXCEEDED",
            Self::CapabilityDenied => "RC-CAPABILITY-DENIED",
            Self::KeyUnknown => "RC-KEY-UNKNOWN",
            Self::CtxInvalid => "RC-CTX-INVALID",
            Self::PolicyDenied => "RC-POLICY-DENIED",
            Self::AdapterFailed => "RC-ADAPTER-FAILED",
            Self::NotImplemented => "RC-NOT-IMPLEMENTED",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: ErrorCode,
    pub phase: DiagPhase,
    pub span: Span,
    pub message: String,
    pub hint: Option<String>,
}

impl Diagnostic {
    pub fn new(code: ErrorCode, phase: DiagPhase, span: Span, message: impl Into<String>) -> Self {
        Self {
            code,
            phase,
            span,
            message: message.into(),
            hint: None,
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}
