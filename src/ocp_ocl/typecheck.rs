use std::collections::HashMap;

use crate::ocp_ocl::ast::{Expr, Program, Stmt};
use crate::ocp_ocl::diag::{DiagPhase, Diagnostic, ErrorCode};
use crate::ocp_ocl::span::Span;
use crate::ocp_ocl::types::Type;

#[derive(Debug, Clone)]
struct VarInfo {
    ty: Type,
    from_observe: bool,
}

pub fn typecheck_program(program: &Program) -> Result<(), Diagnostic> {
    TypeChecker::new().check_program(program)
}

pub struct TypeChecker {
    vars: HashMap<String, VarInfo>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    pub fn check_program(&mut self, program: &Program) -> Result<(), Diagnostic> {
        for stmt in &program.statements {
            self.check_stmt(stmt)?;
        }
        Ok(())
    }

    fn check_stmt(&mut self, stmt: &Stmt) -> Result<(), Diagnostic> {
        match stmt {
            Stmt::Let {
                name,
                value,
                span: _,
            } => {
                let ty = self.infer_expr(value)?;
                self.vars.insert(
                    name.clone(),
                    VarInfo {
                        ty,
                        from_observe: false,
                    },
                );
                Ok(())
            }
            Stmt::Observe {
                key,
                tier,
                ctx,
                budget,
                bind,
                span,
            } => {
                self.expect_expr_type(key, Type::String, *span, "observe key must be string")?;
                self.expect_expr_type(tier, Type::String, *span, "observe tier must be string")?;
                self.expect_expr_type(ctx, Type::Ctx, *span, "observe ctx must be ctx(...)")?;
                self.expect_expr_type(
                    budget,
                    Type::Budget,
                    *span,
                    "observe budget must be budget(...)",
                )?;

                self.vars.insert(
                    bind.clone(),
                    VarInfo {
                        ty: Type::result4_payload(),
                        from_observe: true,
                    },
                );
                Ok(())
            }
            Stmt::Commit { value, span } => {
                self.check_commit_expr(value, *span)?;
                Ok(())
            }
            Stmt::Condition { value, span } => {
                self.expect_expr_type(value, Type::Bool, *span, "condition(...) expects bool")?;
                Ok(())
            }
            Stmt::Entangle {
                left,
                right,
                constraint,
                span,
            } => {
                if !self.vars.contains_key(left) {
                    return Err(self.unknown_ident(*span, left));
                }
                if !self.vars.contains_key(right) {
                    return Err(self.unknown_ident(*span, right));
                }
                self.expect_expr_type(
                    constraint,
                    Type::Bool,
                    *span,
                    "entangle(...) constraint must be bool",
                )?;
                Ok(())
            }
            Stmt::Match(m) => {
                let t = self.infer_expr(&m.value)?;
                if !matches!(t, Type::Result4(_)) {
                    return Err(self
                        .type_error(m.span, format!("match expects Result4, got {}", t.as_str())));
                }
                self.check_block(&m.ok_arm)?;
                self.check_block(&m.degraded_arm)?;
                self.check_block(&m.insufficient_arm)?;
                self.check_block(&m.deferred_arm)?;
                Ok(())
            }
        }
    }

    fn check_block(&mut self, stmts: &[Stmt]) -> Result<(), Diagnostic> {
        let snapshot = self.vars.clone();
        for stmt in stmts {
            self.check_stmt(stmt)?;
        }
        self.vars = snapshot;
        Ok(())
    }

    fn check_commit_expr(&mut self, expr: &Expr, span: Span) -> Result<(), Diagnostic> {
        match expr {
            Expr::Ident {
                name,
                span: id_span,
            } => {
                let Some(info) = self.vars.get(name) else {
                    return Err(self.unknown_ident(*id_span, name));
                };
                if !matches!(info.ty, Type::Result4(_)) || !info.from_observe {
                    return Err(self.type_error(
                        span,
                        "commit(...) expects identifier bound from observe(...)",
                    ));
                }
                Ok(())
            }
            _ => Err(self.type_error(
                span,
                "commit(...) expects identifier bound from observe(...)",
            )),
        }
    }

    fn expect_expr_type(
        &mut self,
        expr: &Expr,
        expected: Type,
        span: Span,
        msg: &str,
    ) -> Result<(), Diagnostic> {
        let got = self.infer_expr(expr)?;
        if got != expected {
            return Err(self.type_error(span, format!("{msg}; got {}", got.as_str())));
        }
        Ok(())
    }

    fn infer_expr(&mut self, expr: &Expr) -> Result<Type, Diagnostic> {
        match expr {
            Expr::Int { .. } => Ok(Type::Int),
            Expr::Bool { .. } => Ok(Type::Bool),
            Expr::String { .. } => Ok(Type::String),
            Expr::Ident { name, span } => {
                let Some(info) = self.vars.get(name) else {
                    return Err(self.unknown_ident(*span, name));
                };
                Ok(info.ty.clone())
            }
            Expr::Call { callee, args, span } => self.infer_call(callee, args, *span),
            Expr::FieldAccess {
                base,
                field: _,
                span: _,
            } => {
                let base_ty = self.infer_expr(base)?;
                match base_ty {
                    Type::Payload | Type::Unknown => Ok(Type::Unknown),
                    Type::Result4(inner) if *inner == Type::Payload => Ok(Type::Unknown),
                    other => Err(self.type_error(
                        expr.span(),
                        format!("field access is not allowed on type {}", other.as_str()),
                    )),
                }
            }
        }
    }

    fn infer_call(&mut self, callee: &str, args: &[Expr], span: Span) -> Result<Type, Diagnostic> {
        match callee {
            "budget" => {
                if args.len() != 1 {
                    return Err(self.type_error(span, "budget(...) expects exactly 1 argument"));
                }
                let arg_ty = self.infer_expr(&args[0])?;
                if arg_ty != Type::Int {
                    return Err(self.type_error(span, "budget(...) argument must be int"));
                }
                Ok(Type::Budget)
            }
            "ctx" => {
                if args.len() != 1 {
                    return Err(self.type_error(span, "ctx(...) expects exactly 1 argument"));
                }
                let arg_ty = self.infer_expr(&args[0])?;
                if arg_ty != Type::String {
                    return Err(self.type_error(span, "ctx(...) argument must be string"));
                }
                Ok(Type::Ctx)
            }
            _ => Err(Diagnostic::new(
                ErrorCode::TUnknownIdentifier,
                DiagPhase::Typecheck,
                span,
                format!("unknown call target `{callee}`"),
            )),
        }
    }

    fn unknown_ident(&self, span: Span, name: &str) -> Diagnostic {
        Diagnostic::new(
            ErrorCode::TUnknownIdentifier,
            DiagPhase::Typecheck,
            span,
            format!("unknown identifier `{name}`"),
        )
    }

    fn type_error(&self, span: Span, message: impl Into<String>) -> Diagnostic {
        Diagnostic::new(
            ErrorCode::TTypeMismatch,
            DiagPhase::Typecheck,
            span,
            message,
        )
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}
