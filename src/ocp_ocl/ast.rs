use crate::ocp_ocl::Span;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    Let {
        name: String,
        value: Expr,
        span: Span,
    },
    Observe {
        key: Expr,
        tier: Expr,
        ctx: Expr,
        budget: Expr,
        bind: String,
        span: Span,
    },
    Commit {
        value: Expr,
        span: Span,
    },
    Condition {
        value: Expr,
        span: Span,
    },
    Match(MatchStmt),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchStmt {
    pub value: Expr,
    pub ok_arm: Vec<Stmt>,
    pub degraded_arm: Vec<Stmt>,
    pub insufficient_arm: Vec<Stmt>,
    pub deferred_arm: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Int {
        value: i64,
        span: Span,
    },
    Bool {
        value: bool,
        span: Span,
    },
    String {
        value: String,
        span: Span,
    },
    Ident {
        name: String,
        span: Span,
    },
    Call {
        callee: String,
        args: Vec<Expr>,
        span: Span,
    },
    FieldAccess {
        base: Box<Expr>,
        field: String,
        span: Span,
    },
}

impl Expr {
    pub const fn span(&self) -> Span {
        match self {
            Self::Int { span, .. }
            | Self::Bool { span, .. }
            | Self::String { span, .. }
            | Self::Ident { span, .. }
            | Self::Call { span, .. }
            | Self::FieldAccess { span, .. } => *span,
        }
    }
}
