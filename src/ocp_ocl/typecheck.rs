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

#[derive(Debug, Clone)]
struct FunctionSig {
    params: Vec<String>,
    return_ty: Type,
}

pub fn typecheck_program(program: &Program) -> Result<(), Diagnostic> {
    TypeChecker::new().check_program(program)
}

pub struct TypeChecker {
    vars: HashMap<String, VarInfo>,
    funcs: HashMap<String, FunctionSig>,
    in_function: bool,
    current_return_ty: Option<Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            funcs: HashMap::new(),
            in_function: false,
            current_return_ty: None,
        }
    }

    pub fn check_program(&mut self, program: &Program) -> Result<(), Diagnostic> {
        for stmt in &program.statements {
            if let Stmt::FnDef {
                name,
                params,
                span: _,
                ..
            } = stmt
            {
                self.funcs.insert(
                    name.clone(),
                    FunctionSig {
                        params: params.clone(),
                        return_ty: Type::Unknown,
                    },
                );
            }
        }

        for stmt in &program.statements {
            self.check_stmt(stmt)?;
        }
        Ok(())
    }

    fn check_stmt(&mut self, stmt: &Stmt) -> Result<(), Diagnostic> {
        match stmt {
            Stmt::ModuleDecl { .. }
            | Stmt::ImportDecl { .. }
            | Stmt::ExportDecl { .. }
            | Stmt::StructDecl { .. }
            | Stmt::EnumDecl { .. } => Ok(()),
            Stmt::FnDef {
                name,
                params,
                body,
                span,
            } => self.check_fn_def(name, params, body, *span),
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
            Stmt::Return { value, span } => self.check_return_stmt(value, *span),
            Stmt::ForRange {
                var,
                start,
                end,
                body,
                span,
            } => self.check_for_range(var, start, end, body, *span),
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

    fn check_fn_def(
        &mut self,
        name: &str,
        params: &[String],
        body: &[Stmt],
        span: Span,
    ) -> Result<(), Diagnostic> {
        let snapshot = self.vars.clone();
        let in_fn_before = self.in_function;
        let ret_before = self.current_return_ty.clone();

        self.in_function = true;
        self.current_return_ty = None;

        for param in params {
            self.vars.insert(
                param.clone(),
                VarInfo {
                    ty: Type::Unknown,
                    from_observe: false,
                },
            );
        }

        for stmt in body {
            self.check_stmt(stmt)?;
        }

        let return_ty = self.current_return_ty.clone().unwrap_or(Type::Unit);
        if let Some(sig) = self.funcs.get_mut(name) {
            sig.return_ty = return_ty;
        } else {
            return Err(self.type_error(
                span,
                format!("internal typecheck error: function `{name}` not registered"),
            ));
        }

        self.vars = snapshot;
        self.in_function = in_fn_before;
        self.current_return_ty = ret_before;
        Ok(())
    }

    fn check_return_stmt(&mut self, value: &Expr, span: Span) -> Result<(), Diagnostic> {
        if !self.in_function {
            return Err(self.type_error(span, "return is only allowed inside function body"));
        }
        let ty = self.infer_expr(value)?;
        match &self.current_return_ty {
            None => {
                self.current_return_ty = Some(ty);
                Ok(())
            }
            Some(current) if *current == ty || *current == Type::Unknown || ty == Type::Unknown => {
                Ok(())
            }
            Some(current) => Err(self.type_error(
                span,
                format!(
                    "inconsistent return type in function body: expected {}, got {}",
                    current.as_str(),
                    ty.as_str()
                ),
            )),
        }
    }

    fn check_for_range(
        &mut self,
        var: &str,
        start: &Expr,
        end: &Expr,
        body: &[Stmt],
        span: Span,
    ) -> Result<(), Diagnostic> {
        self.expect_expr_type(start, Type::Int, span, "for-range start must be int")?;
        self.expect_expr_type(end, Type::Int, span, "for-range end must be int")?;
        let Expr::Int { value: end_val, .. } = end else {
            return Err(
                self.type_error(span, "for-range end must be bounded int literal in v0.3 M1")
            );
        };
        if *end_val < 0 || *end_val > 10_000 {
            return Err(self.type_error(
                span,
                "for-range end literal is out of allowed bound (0..=10000)",
            ));
        }

        let snapshot = self.vars.clone();
        self.vars.insert(
            var.to_string(),
            VarInfo {
                ty: Type::Int,
                from_observe: false,
            },
        );
        for stmt in body {
            self.check_stmt(stmt)?;
        }
        self.vars = snapshot;
        Ok(())
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
            Expr::List { items, span: _ } => {
                if items.is_empty() {
                    return Ok(Type::List(Box::new(Type::Unknown)));
                }
                let mut inner = self.infer_expr(&items[0])?;
                for item in items.iter().skip(1) {
                    let item_ty = self.infer_expr(item)?;
                    if inner == Type::Unknown {
                        inner = item_ty;
                    } else if item_ty != Type::Unknown && item_ty != inner {
                        return Err(self.type_error(
                            expr.span(),
                            "list literal contains incompatible item types",
                        ));
                    }
                }
                Ok(Type::List(Box::new(inner)))
            }
            Expr::Map { entries, .. } => {
                if entries.is_empty() {
                    return Ok(Type::Map(Box::new(Type::String), Box::new(Type::Unknown)));
                }
                let mut value_ty = self.infer_expr(&entries[0].1)?;
                for (_, value) in entries.iter().skip(1) {
                    let vt = self.infer_expr(value)?;
                    if value_ty == Type::Unknown {
                        value_ty = vt;
                    } else if vt != Type::Unknown && vt != value_ty {
                        return Err(self.type_error(
                            expr.span(),
                            "map literal contains incompatible value types",
                        ));
                    }
                }
                Ok(Type::Map(Box::new(Type::String), Box::new(value_ty)))
            }
            Expr::Try { value, .. } => {
                let base = self.infer_expr(value)?;
                match base {
                    Type::Result4(inner) => Ok(*inner),
                    _ => Err(self.type_error(
                        expr.span(),
                        format!("`?` expects Result4, got {}", base.as_str()),
                    )),
                }
            }
            Expr::FieldAccess {
                base,
                field: _,
                span: _,
            } => {
                let base_ty = self.infer_expr(base)?;
                match base_ty {
                    Type::Payload | Type::Unknown => Ok(Type::Unknown),
                    Type::Map(_, _) => Ok(Type::Unknown),
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
            "payload" => {
                if !args.is_empty() {
                    return Err(self.type_error(span, "payload() expects 0 arguments"));
                }
                Ok(Type::Payload)
            }
            _ => {
                let Some(sig) = self.funcs.get(callee).cloned() else {
                    return Err(Diagnostic::new(
                        ErrorCode::TUnknownIdentifier,
                        DiagPhase::Typecheck,
                        span,
                        format!("unknown call target `{callee}`"),
                    ));
                };
                if sig.params.len() != args.len() {
                    return Err(self.type_error(
                        span,
                        format!(
                            "function `{callee}` expects {} args, got {}",
                            sig.params.len(),
                            args.len()
                        ),
                    ));
                }
                for arg in args {
                    let _ = self.infer_expr(arg)?;
                }
                Ok(sig.return_ty)
            }
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
