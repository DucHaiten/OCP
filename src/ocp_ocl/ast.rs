use crate::ocp_ocl::Span;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LetPattern {
    Ident(String),
    Record(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    ModuleDecl {
        path: Vec<String>,
        span: Span,
    },
    ImportDecl {
        path: Vec<String>,
        span: Span,
    },
    ExportDecl {
        name: String,
        span: Span,
    },
    StructDecl {
        name: String,
        fields: Vec<String>,
        span: Span,
    },
    EnumDecl {
        name: String,
        variants: Vec<String>,
        span: Span,
    },
    FnDef {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
        span: Span,
    },
    Let {
        pattern: LetPattern,
        value: Expr,
        span: Span,
    },
    Return {
        value: Expr,
        span: Span,
    },
    TryLet {
        name: String,
        value: Expr,
        else_expr: Expr,
        else_returns: bool,
        span: Span,
    },
    Guard {
        value: Expr,
        span: Span,
    },
    Repeat {
        count: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    ForEachCap {
        var: String,
        iter: Expr,
        cap: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    ForRange {
        var: String,
        start: Expr,
        end: Expr,
        body: Vec<Stmt>,
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
    Entangle {
        left: String,
        right: String,
        constraint: Expr,
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
    List {
        items: Vec<Expr>,
        span: Span,
    },
    Map {
        entries: Vec<(String, Expr)>,
        span: Span,
    },
    Record {
        fields: Vec<(String, Expr)>,
        span: Span,
    },
    Try {
        value: Box<Expr>,
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
            | Self::List { span, .. }
            | Self::Map { span, .. }
            | Self::Record { span, .. }
            | Self::Try { span, .. }
            | Self::FieldAccess { span, .. } => *span,
        }
    }
}
