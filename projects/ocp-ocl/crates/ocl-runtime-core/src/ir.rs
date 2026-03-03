use ocp_ocl::ocp_ocl::{MatchStmt, Program, Span, Stmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrOpKind {
    ModuleDecl,
    ImportDecl,
    ExportDecl,
    StructDecl,
    EnumDecl,
    FnDef,
    Let,
    TryLet,
    Guard,
    Return,
    Repeat,
    ForEachCap,
    ForRange,
    Observe,
    Commit,
    Condition,
    Entangle,
    Match,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrOp {
    pub kind: IrOpKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypedIrProgram {
    pub ops: Vec<IrOp>,
}

impl TypedIrProgram {
    pub fn op_count(&self) -> usize {
        self.ops.len()
    }
}

pub fn lower_program(program: &Program) -> TypedIrProgram {
    let mut out = TypedIrProgram::default();
    lower_stmts(&program.statements, &mut out.ops);
    out
}

fn lower_stmts(stmts: &[Stmt], out: &mut Vec<IrOp>) {
    for stmt in stmts {
        match stmt {
            Stmt::ModuleDecl { span, .. } => out.push(IrOp {
                kind: IrOpKind::ModuleDecl,
                span: *span,
            }),
            Stmt::ImportDecl { span, .. } => out.push(IrOp {
                kind: IrOpKind::ImportDecl,
                span: *span,
            }),
            Stmt::ExportDecl { span, .. } => out.push(IrOp {
                kind: IrOpKind::ExportDecl,
                span: *span,
            }),
            Stmt::StructDecl { span, .. } => out.push(IrOp {
                kind: IrOpKind::StructDecl,
                span: *span,
            }),
            Stmt::EnumDecl { span, .. } => out.push(IrOp {
                kind: IrOpKind::EnumDecl,
                span: *span,
            }),
            Stmt::FnDef { span, body, .. } => {
                out.push(IrOp {
                    kind: IrOpKind::FnDef,
                    span: *span,
                });
                lower_stmts(body, out);
            }
            Stmt::Let { span, .. } => out.push(IrOp {
                kind: IrOpKind::Let,
                span: *span,
            }),
            Stmt::TryLet { span, .. } => out.push(IrOp {
                kind: IrOpKind::TryLet,
                span: *span,
            }),
            Stmt::Guard { span, .. } => out.push(IrOp {
                kind: IrOpKind::Guard,
                span: *span,
            }),
            Stmt::Return { span, .. } => out.push(IrOp {
                kind: IrOpKind::Return,
                span: *span,
            }),
            Stmt::Repeat { span, body, .. } => {
                out.push(IrOp {
                    kind: IrOpKind::Repeat,
                    span: *span,
                });
                lower_stmts(body, out);
            }
            Stmt::ForEachCap { span, body, .. } => {
                out.push(IrOp {
                    kind: IrOpKind::ForEachCap,
                    span: *span,
                });
                lower_stmts(body, out);
            }
            Stmt::ForRange { span, body, .. } => {
                out.push(IrOp {
                    kind: IrOpKind::ForRange,
                    span: *span,
                });
                lower_stmts(body, out);
            }
            Stmt::Observe { span, .. } => out.push(IrOp {
                kind: IrOpKind::Observe,
                span: *span,
            }),
            Stmt::Commit { span, .. } => out.push(IrOp {
                kind: IrOpKind::Commit,
                span: *span,
            }),
            Stmt::Condition { span, .. } => out.push(IrOp {
                kind: IrOpKind::Condition,
                span: *span,
            }),
            Stmt::Entangle { span, .. } => out.push(IrOp {
                kind: IrOpKind::Entangle,
                span: *span,
            }),
            Stmt::Match(m) => {
                out.push(IrOp {
                    kind: IrOpKind::Match,
                    span: m.span,
                });
                lower_match(m, out);
            }
        }
    }
}

fn lower_match(m: &MatchStmt, out: &mut Vec<IrOp>) {
    lower_stmts(&m.ok_arm, out);
    lower_stmts(&m.degraded_arm, out);
    lower_stmts(&m.insufficient_arm, out);
    lower_stmts(&m.deferred_arm, out);
}
