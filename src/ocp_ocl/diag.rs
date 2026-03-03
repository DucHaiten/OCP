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
    XConditionFalse,
    XConditionDeferred,
    XConditionInsufficient,
    XEntangleBindingUnknown,
    XEntangleConstraintType,
    XEntangleConstraintFalse,
    XEntangleEdgeCap,
    XEntangleDegreeCap,
    XEntangleDeferred,
    XBudgetExceeded,
    XCommitForbidden,
    RCapabilityDenied,
    RCtxInvalid,
    EConditionFalse,
    EBudgetExceeded,
    ECommitForbidden,
    ECapabilityDenied,
}

impl ErrorCode {
    pub const fn canonical(self) -> Self {
        match self {
            Self::EConditionFalse => Self::XConditionFalse,
            Self::EBudgetExceeded => Self::XBudgetExceeded,
            Self::ECommitForbidden => Self::XCommitForbidden,
            Self::ECapabilityDenied => Self::RCapabilityDenied,
            other => other,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self.canonical() {
            Self::PUnexpectedToken => "P-UNEXPECTED-TOKEN",
            Self::PUnexpectedEof => "P-UNEXPECTED-EOF",
            Self::PInvalidLiteral => "P-INVALID-LITERAL",
            Self::TUnknownIdentifier => "T-UNKNOWN-IDENTIFIER",
            Self::TTypeMismatch => "T-TYPE-MISMATCH",
            Self::TMatchArmsIncomplete => "T-MATCH-ARMS-INCOMPLETE",
            Self::XConditionFalse => "X-COND-FALSE",
            Self::XConditionDeferred => "X-COND-DEFERRED",
            Self::XConditionInsufficient => "X-COND-INSUFFICIENT",
            Self::XEntangleBindingUnknown => "X-ENTANGLE-BINDING-UNKNOWN",
            Self::XEntangleConstraintType => "X-ENTANGLE-CONSTRAINT-TYPE",
            Self::XEntangleConstraintFalse => "X-ENTANGLE-CONSTRAINT-FALSE",
            Self::XEntangleEdgeCap => "X-ENTANGLE-EDGE-CAP",
            Self::XEntangleDegreeCap => "X-ENTANGLE-DEGREE-CAP",
            Self::XEntangleDeferred => "X-ENTANGLE-DEFERRED",
            Self::XBudgetExceeded => "X-BUDGET-EXCEEDED",
            Self::XCommitForbidden => "X-COMMIT-FORBIDDEN",
            Self::RCapabilityDenied => "R-CAPABILITY-DENIED",
            Self::RCtxInvalid => "R-CTX-INVALID",
            Self::EConditionFalse => "X-COND-FALSE",
            Self::EBudgetExceeded => "X-BUDGET-EXCEEDED",
            Self::ECommitForbidden => "X-COMMIT-FORBIDDEN",
            Self::ECapabilityDenied => "R-CAPABILITY-DENIED",
        }
    }

    pub const fn legacy_alias(self) -> Option<&'static str> {
        match self.canonical() {
            Self::XConditionFalse => Some("E-CONDITION-FALSE"),
            Self::XBudgetExceeded => Some("E-BUDGET-EXCEEDED"),
            Self::XCommitForbidden => Some("E-COMMIT-FORBIDDEN"),
            Self::RCapabilityDenied => Some("E-CAPABILITY-DENIED"),
            _ => None,
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
    PluginProtocolError,
    PluginUnavailable,
    PluginTimeout,
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
            Self::PluginProtocolError => "RC-PLUGIN-PROTOCOL-ERROR",
            Self::PluginUnavailable => "RC-PLUGIN-UNAVAILABLE",
            Self::PluginTimeout => "RC-PLUGIN-TIMEOUT",
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
    pub root_reason: Option<ReasonCode>,
}

impl Diagnostic {
    pub fn new(code: ErrorCode, phase: DiagPhase, span: Span, message: impl Into<String>) -> Self {
        Self {
            code,
            phase,
            span,
            message: message.into(),
            hint: None,
            root_reason: None,
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn with_root_reason(mut self, reason: ReasonCode) -> Self {
        self.root_reason = Some(reason);
        self
    }
}
