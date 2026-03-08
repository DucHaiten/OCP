use crate::ocp::span::Span;

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
    TImportNotFound,
    TImportCycle,
    TForNotList,
    TCapNotIntLit,
    TTryNotResult4,
    TGuardNotResult4,
    TTryElseNoValue,
    TCommitForbiddenKey,
    TCtxMissingField,
    TCtxUnknownField,
    TCtxTypeMismatch,
    TCtxConstraintViolation,
    TCtxStringCompat,
    TKeyNotLiteralForSchema,
    XConditionFalse,
    XConditionDeferred,
    XConditionInsufficient,
    XEntangleBindingUnknown,
    XEntangleConstraintType,
    XEntangleConstraintFalse,
    XEntangleEdgeCap,
    XEntangleDegreeCap,
    XEntangleDeferred,
    XLoopCapExceeded,
    XKeysCapExceeded,
    XGuardFailed,
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
            Self::TImportNotFound => "T-IMPORT-NOT-FOUND",
            Self::TImportCycle => "T-IMPORT-CYCLE",
            Self::TForNotList => "T-FOR-NOT-LIST",
            Self::TCapNotIntLit => "T-CAP-NOT-INT-LIT",
            Self::TTryNotResult4 => "T-TRY-NOT-RESULT4",
            Self::TGuardNotResult4 => "T-GUARD-NOT-RESULT4",
            Self::TTryElseNoValue => "T-TRY-ELSE-NO-VALUE",
            Self::TCommitForbiddenKey => "T-COMMIT-FORBIDDEN-KEY",
            Self::TCtxMissingField => "T-CTX-MISSING-FIELD",
            Self::TCtxUnknownField => "T-CTX-UNKNOWN-FIELD",
            Self::TCtxTypeMismatch => "T-CTX-TYPE-MISMATCH",
            Self::TCtxConstraintViolation => "T-CTX-CONSTRAINT-VIOLATION",
            Self::TCtxStringCompat => "T-CTX-STRING-COMPAT",
            Self::TKeyNotLiteralForSchema => "T-KEY-NOT-LITERAL-FOR-SCHEMA",
            Self::XConditionFalse => "X-COND-FALSE",
            Self::XConditionDeferred => "X-COND-DEFERRED",
            Self::XConditionInsufficient => "X-COND-INSUFFICIENT",
            Self::XEntangleBindingUnknown => "X-ENTANGLE-BINDING-UNKNOWN",
            Self::XEntangleConstraintType => "X-ENTANGLE-CONSTRAINT-TYPE",
            Self::XEntangleConstraintFalse => "X-ENTANGLE-CONSTRAINT-FALSE",
            Self::XEntangleEdgeCap => "X-ENTANGLE-EDGE-CAP",
            Self::XEntangleDegreeCap => "X-ENTANGLE-DEGREE-CAP",
            Self::XEntangleDeferred => "X-ENTANGLE-DEFERRED",
            Self::XLoopCapExceeded => "X-LOOP-CAP-EXCEEDED",
            Self::XKeysCapExceeded => "X-KEYS-CAP-EXCEEDED",
            Self::XGuardFailed => "X-GUARD-FAILED",
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
    FsNotFound,
    FsPermissionDenied,
    FsPathOutsideSandbox,
    FsSymlinkDisallowed,
    FsInvalidPath,
    FsTooLarge,
    FsIoError,
    KvNotFound,
    KvPermissionDenied,
    KvCapExceeded,
    KvIoError,
    TimeDisabled,
    GameDisabled,
    GameRngInvalidStream,
    ShadowDisabled,
    ShadowCapExceeded,
    ShadowPolicyDenied,
    ShadowReportTooLarge,
    ShadowEffectDisallowed,
    UiDisabled,
    UiCapExceeded,
    QuarantineRequired,
    CassetteMissing,
    CassetteMiss,
    CassetteTooLarge,
    NetInvalidUrl,
    NetDnsFail,
    NetTimeout,
    NetTlsFail,
    NetHostDenied,
    NetMethodDenied,
    ProcBinDenied,
    ProcTimeout,
    ProcNotFound,
    ProcExecFail,
    LimitExceeded,
    JsonInvalid,
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
            Self::FsNotFound => "RC-FS-NOT-FOUND",
            Self::FsPermissionDenied => "RC-FS-PERMISSION-DENIED",
            Self::FsPathOutsideSandbox => "RC-FS-PATH-OUTSIDE-SANDBOX",
            Self::FsSymlinkDisallowed => "RC-FS-SYMLINK-DISALLOWED",
            Self::FsInvalidPath => "RC-FS-INVALID-PATH",
            Self::FsTooLarge => "RC-FS-TOO-LARGE",
            Self::FsIoError => "RC-FS-IO-ERROR",
            Self::KvNotFound => "RC-KV-NOT-FOUND",
            Self::KvPermissionDenied => "RC-KV-PERMISSION-DENIED",
            Self::KvCapExceeded => "RC-KV-CAP-EXCEEDED",
            Self::KvIoError => "RC-KV-IO-ERROR",
            Self::TimeDisabled => "RC-TIME-DISABLED",
            Self::GameDisabled => "RC-GAME-DISABLED",
            Self::GameRngInvalidStream => "RC-GAME-RNG-INVALID-STREAM",
            Self::ShadowDisabled => "RC-SHADOW-DISABLED",
            Self::ShadowCapExceeded => "RC-SHADOW-CAP-EXCEEDED",
            Self::ShadowPolicyDenied => "RC-SHADOW-POLICY-DENIED",
            Self::ShadowReportTooLarge => "RC-SHADOW-REPORT-TOO-LARGE",
            Self::ShadowEffectDisallowed => "RC-SHADOW-EFFECT-DISALLOWED",
            Self::UiDisabled => "RC-UI-DISABLED",
            Self::UiCapExceeded => "RC-UI-CAP-EXCEEDED",
            Self::QuarantineRequired => "RC-QUARANTINE-REQUIRED",
            Self::CassetteMissing => "RC-CASSETTE-MISSING",
            Self::CassetteMiss => "RC-CASSETTE-MISS",
            Self::CassetteTooLarge => "RC-CASSETTE-TOO-LARGE",
            Self::NetInvalidUrl => "RC-NET-INVALID-URL",
            Self::NetDnsFail => "RC-NET-DNS-FAIL",
            Self::NetTimeout => "RC-NET-TIMEOUT",
            Self::NetTlsFail => "RC-NET-TLS-FAIL",
            Self::NetHostDenied => "RC-NET-HOST-DENIED",
            Self::NetMethodDenied => "RC-NET-METHOD-DENIED",
            Self::ProcBinDenied => "RC-PROC-BIN-DENIED",
            Self::ProcTimeout => "RC-PROC-TIMEOUT",
            Self::ProcNotFound => "RC-PROC-NOT-FOUND",
            Self::ProcExecFail => "RC-PROC-EXEC-FAIL",
            Self::LimitExceeded => "RC-LIMIT-EXCEEDED",
            Self::JsonInvalid => "RC-JSON-INVALID",
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
    pub aliases: Vec<String>,
    pub limit_kind: Option<String>,
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
            aliases: Vec::new(),
            limit_kind: None,
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

    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        if !self.aliases.iter().any(|v| v == &alias) {
            self.aliases.push(alias);
        }
        self
    }

    pub fn with_limit_kind(mut self, limit_kind: impl Into<String>) -> Self {
        self.limit_kind = Some(limit_kind.into());
        self
    }
}
