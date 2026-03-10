use std::collections::{BTreeMap, HashMap, HashSet};

use crate::ocp::ast::{Expr, LetPattern, Program, Stmt};
use crate::ocp::diag::{DiagPhase, Diagnostic, ErrorCode};
use crate::ocp::registry::CapabilityRegistry;
use crate::ocp::schema::{schema_skeleton, validate_schema_value, SchemaIssueCode, SchemaType};
use crate::ocp::span::Span;
use crate::ocp::types::Type;
use crate::ocp::value::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatMode {
    Allow,
    Warn,
    Deny,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypecheckCompatConfig {
    pub ctx_string: CompatMode,
    pub ctx_extra_fields: CompatMode,
}

impl Default for TypecheckCompatConfig {
    fn default() -> Self {
        Self {
            ctx_string: CompatMode::Warn,
            ctx_extra_fields: CompatMode::Deny,
        }
    }
}

#[derive(Debug, Clone)]
struct VarInfo {
    ty: Type,
    from_observe: bool,
    observe_key_literal: Option<String>,
    observe_commit_allowed: Option<bool>,
}

#[derive(Debug, Clone)]
struct FunctionSig {
    params: Vec<String>,
    return_ty: Type,
}

pub fn typecheck_program(program: &Program) -> Result<(), Diagnostic> {
    typecheck_program_with_compat(program, TypecheckCompatConfig::default())
}

pub fn typecheck_program_with_compat(
    program: &Program,
    compat: TypecheckCompatConfig,
) -> Result<(), Diagnostic> {
    TypeChecker::with_compat(compat).check_program(program)
}

pub struct TypeChecker {
    vars: HashMap<String, VarInfo>,
    funcs: HashMap<String, FunctionSig>,
    registry: CapabilityRegistry,
    in_function: bool,
    current_return_ty: Option<Type>,
    compat: TypecheckCompatConfig,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self::with_compat(TypecheckCompatConfig::default())
    }

    pub fn with_compat(compat: TypecheckCompatConfig) -> Self {
        Self {
            vars: HashMap::new(),
            funcs: HashMap::new(),
            registry: CapabilityRegistry::default(),
            in_function: false,
            current_return_ty: None,
            compat,
        }
    }

    pub fn with_registry(registry: CapabilityRegistry) -> Self {
        Self::with_registry_and_compat(registry, TypecheckCompatConfig::default())
    }

    pub fn with_registry_and_compat(
        registry: CapabilityRegistry,
        compat: TypecheckCompatConfig,
    ) -> Self {
        Self {
            vars: HashMap::new(),
            funcs: HashMap::new(),
            registry,
            in_function: false,
            current_return_ty: None,
            compat,
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
                pattern,
                value,
                span,
            } => {
                let ty = self.infer_expr(value)?;
                self.bind_let_pattern(pattern, ty, *span)?;
                Ok(())
            }
            Stmt::Return { value, span } => self.check_return_stmt(value, *span),
            Stmt::TryLet {
                name,
                value,
                else_expr,
                else_returns,
                span,
            } => self.check_try_let(name, value, else_expr, *else_returns, *span),
            Stmt::Guard { value, span } => self.check_guard(value, *span),
            Stmt::Repeat { count, body, span } => self.check_repeat(count, body, *span),
            Stmt::ForEachCap {
                var,
                iter,
                cap,
                body,
                span,
            } => self.check_for_each_cap(var, iter, cap, body, *span),
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
                if let Expr::String { value, .. } = key {
                    if value == "std.net.poll" {
                        return Err(self.type_error(
                            *span,
                            "`std.net.poll` is runtime-only and forbidden in user OCP path",
                        ));
                    }
                }

                self.expect_expr_type(key, Type::String, *span, "observe key must be string")?;
                self.expect_expr_type(tier, Type::String, *span, "observe tier must be string")?;
                let ctx_ty = self.infer_expr(ctx)?;
                if !matches!(
                    ctx_ty,
                    Type::Ctx | Type::Record(_) | Type::Map(_, _) | Type::Unknown
                ) {
                    return Err(self.type_error(
                        *span,
                        format!(
                            "observe ctx must be ctx(...)/record/map-compatible value; got {}",
                            ctx_ty.as_str()
                        ),
                    ));
                }
                self.expect_expr_type(
                    budget,
                    Type::Budget,
                    *span,
                    "observe budget must be budget(...)",
                )?;

                if let Expr::String { value: key_lit, .. } = key {
                    if matches!(ctx_ty, Type::Ctx)
                        && self.registry.ctx_schema_for_key(key_lit).is_some()
                        && self.compat.ctx_string == CompatMode::Deny
                    {
                        return Err(Diagnostic::new(
                            ErrorCode::TCtxStringCompat,
                            DiagPhase::Typecheck,
                            *span,
                            format!(
                                "ctx(\"...\") is denied by compatibility policy for `{key_lit}`"
                            ),
                        )
                        .with_hint(
                            "use typed record ctx, or set `[compat].ctx_string = \"allow|warn\"`"
                                .to_string(),
                        ));
                    }
                    self.validate_observe_ctx_schema(key_lit, ctx, &ctx_ty, *span)?;
                }

                let (observe_key_literal, observe_commit_allowed) =
                    if let Expr::String { value: key_lit, .. } = key {
                        (
                            Some(key_lit.clone()),
                            Some(self.registry.commit_allowed_for_key(key_lit)),
                        )
                    } else {
                        (None, None)
                    };

                self.vars.insert(
                    bind.clone(),
                    VarInfo {
                        ty: Type::result4_payload(),
                        from_observe: true,
                        observe_key_literal,
                        observe_commit_allowed,
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

    fn bind_let_pattern(
        &mut self,
        pattern: &LetPattern,
        value_ty: Type,
        span: Span,
    ) -> Result<(), Diagnostic> {
        match pattern {
            LetPattern::Ident(name) => {
                self.vars.insert(
                    name.clone(),
                    VarInfo {
                        ty: value_ty,
                        from_observe: false,
                        observe_key_literal: None,
                        observe_commit_allowed: None,
                    },
                );
                Ok(())
            }
            LetPattern::Record(fields) => {
                let mut seen = HashSet::new();
                for field in fields {
                    if !seen.insert(field) {
                        return Err(self.type_error(
                            span,
                            format!("duplicate field `{field}` in let destructure pattern"),
                        ));
                    }
                }

                match value_ty {
                    Type::Record(map) => {
                        for field in fields {
                            let Some(field_ty) = map.get(field).cloned() else {
                                return Err(self.type_error(
                                    span,
                                    format!(
                                        "let destructure field `{field}` is missing in record value"
                                    ),
                                ));
                            };
                            self.vars.insert(
                                field.clone(),
                                VarInfo {
                                    ty: field_ty,
                                    from_observe: false,
                                    observe_key_literal: None,
                                    observe_commit_allowed: None,
                                },
                            );
                        }
                        Ok(())
                    }
                    Type::Map(_, value_ty) => {
                        for field in fields {
                            self.vars.insert(
                                field.clone(),
                                VarInfo {
                                    ty: (*value_ty).clone(),
                                    from_observe: false,
                                    observe_key_literal: None,
                                    observe_commit_allowed: None,
                                },
                            );
                        }
                        Ok(())
                    }
                    Type::Unknown => {
                        for field in fields {
                            self.vars.insert(
                                field.clone(),
                                VarInfo {
                                    ty: Type::Unknown,
                                    from_observe: false,
                                    observe_key_literal: None,
                                    observe_commit_allowed: None,
                                },
                            );
                        }
                        Ok(())
                    }
                    other => Err(self.type_error(
                        span,
                        format!(
                            "let destructure expects record/map-compatible value, got {}",
                            other.as_str()
                        ),
                    )),
                }
            }
        }
    }

    fn validate_observe_ctx_schema(
        &self,
        key_lit: &str,
        ctx_expr: &Expr,
        ctx_ty: &Type,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let Some(schema) = self.registry.ctx_schema_for_key(key_lit) else {
            return Ok(());
        };

        if let Some(literal_ctx) = expr_literal_to_value(ctx_expr) {
            if let Err(issues) = validate_schema_value(schema, &literal_ctx) {
                let filtered_issues: Vec<_> = issues
                    .into_iter()
                    .filter(|issue| {
                        issue.code != SchemaIssueCode::UnknownField
                            || self.compat.ctx_extra_fields == CompatMode::Deny
                    })
                    .collect();
                if filtered_issues.is_empty() {
                    return Ok(());
                }
                let first = &filtered_issues[0];
                let mut diag = Diagnostic::new(
                    issue_code_to_error_code(first.code),
                    DiagPhase::Typecheck,
                    span,
                    format!("ctx schema mismatch for `{key_lit}` at {}", first.path),
                )
                .with_hint(format!(
                    "{}; expected schema skeleton: {}",
                    first.message,
                    schema_skeleton(schema)
                ));
                if let (Some(expected), Some(got)) = (&first.expected, &first.got) {
                    diag.message = format!(
                        "ctx schema mismatch for `{key_lit}` at {}: expected {}, got {}",
                        first.path, expected, got
                    );
                }
                return Err(diag);
            }
            return Ok(());
        }

        if let Type::Record(fields) = ctx_ty {
            if let SchemaType::Record {
                fields: schema_fields,
                open_row,
            } = schema
            {
                for (name, spec) in schema_fields {
                    if spec.required && !fields.contains_key(name) {
                        return Err(Diagnostic::new(
                            ErrorCode::TCtxMissingField,
                            DiagPhase::Typecheck,
                            span,
                            format!("ctx for `{key_lit}` is missing required field `{name}`"),
                        )
                        .with_hint(format!(
                            "expected schema skeleton: {}",
                            schema_skeleton(schema)
                        )));
                    }
                }
                if !open_row {
                    for field in fields.keys() {
                        if !schema_fields.contains_key(field)
                            && self.compat.ctx_extra_fields == CompatMode::Deny
                        {
                            return Err(Diagnostic::new(
                                ErrorCode::TCtxUnknownField,
                                DiagPhase::Typecheck,
                                span,
                                format!("ctx for `{key_lit}` contains unknown field `{field}`"),
                            )
                            .with_hint(format!(
                                "expected schema skeleton: {}",
                                schema_skeleton(schema)
                            )));
                        }
                    }
                }
                for (name, field_ty) in fields {
                    let Some(spec) = schema_fields.get(name) else {
                        continue;
                    };
                    if !schema_type_compatible_with_type(&spec.ty, field_ty) {
                        return Err(Diagnostic::new(
                            ErrorCode::TCtxTypeMismatch,
                            DiagPhase::Typecheck,
                            span,
                            format!(
                                "ctx field `{name}` for `{key_lit}` has incompatible type {}",
                                field_ty.as_str()
                            ),
                        )
                        .with_hint(format!(
                            "expected schema skeleton: {}",
                            schema_skeleton(schema)
                        )));
                    }
                }
            }
        }

        Ok(())
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
                    observe_key_literal: None,
                    observe_commit_allowed: None,
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
        let ty = self.infer_expr(value)?;
        if !self.in_function {
            // v0.7.1 allows return-from-program at top-level.
            return Ok(());
        }
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
                observe_key_literal: None,
                observe_commit_allowed: None,
            },
        );
        for stmt in body {
            self.check_stmt(stmt)?;
        }
        self.vars = snapshot;
        Ok(())
    }

    fn check_try_let(
        &mut self,
        name: &str,
        value: &Expr,
        else_expr: &Expr,
        else_returns: bool,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let value_ty = self.infer_expr(value)?;
        let inner_ty = match value_ty {
            Type::Result4(inner) => *inner,
            other => {
                return Err(Diagnostic::new(
                    ErrorCode::TTryNotResult4,
                    DiagPhase::Typecheck,
                    span,
                    format!("try/else expects Result4 input, got {}", other.as_str()),
                ));
            }
        };

        let snapshot = self.vars.clone();
        self.vars.insert(
            "r".to_string(),
            VarInfo {
                ty: Type::Result4(Box::new(inner_ty.clone())),
                from_observe: false,
                observe_key_literal: None,
                observe_commit_allowed: None,
            },
        );
        let else_ty = self.infer_expr(else_expr)?;
        self.vars = snapshot;

        if else_returns {
            self.check_return_stmt(else_expr, else_expr.span())?;
            self.vars.insert(
                name.to_string(),
                VarInfo {
                    ty: inner_ty,
                    from_observe: false,
                    observe_key_literal: None,
                    observe_commit_allowed: None,
                },
            );
            return Ok(());
        }

        let final_ty = if type_compatible(&inner_ty, &else_ty) {
            inner_ty
        } else {
            Type::Unknown
        };
        self.vars.insert(
            name.to_string(),
            VarInfo {
                ty: final_ty,
                from_observe: false,
                observe_key_literal: None,
                observe_commit_allowed: None,
            },
        );
        Ok(())
    }

    fn check_guard(&mut self, value: &Expr, span: Span) -> Result<(), Diagnostic> {
        let t = self.infer_expr(value)?;
        if !matches!(t, Type::Result4(_)) {
            return Err(Diagnostic::new(
                ErrorCode::TGuardNotResult4,
                DiagPhase::Typecheck,
                span,
                format!("guard expects Result4 value, got {}", t.as_str()),
            ));
        }
        Ok(())
    }

    fn check_repeat(&mut self, count: &Expr, body: &[Stmt], span: Span) -> Result<(), Diagnostic> {
        self.expect_expr_type(count, Type::Int, span, "repeat count must be int")?;
        self.check_block(body)
    }

    fn check_for_each_cap(
        &mut self,
        var: &str,
        iter: &Expr,
        cap: &Expr,
        body: &[Stmt],
        span: Span,
    ) -> Result<(), Diagnostic> {
        let iter_ty = self.infer_expr(iter)?;
        let item_ty = match iter_ty {
            Type::List(inner) => *inner,
            other => {
                return Err(Diagnostic::new(
                    ErrorCode::TForNotList,
                    DiagPhase::Typecheck,
                    span,
                    format!("for ... cap expects list iterable, got {}", other.as_str()),
                ));
            }
        };
        match cap {
            Expr::Int { value, .. } if *value >= 0 => {}
            _ => {
                return Err(Diagnostic::new(
                    ErrorCode::TCapNotIntLit,
                    DiagPhase::Typecheck,
                    cap.span(),
                    "for ... cap requires non-negative int literal",
                ));
            }
        }

        let snapshot = self.vars.clone();
        self.vars.insert(
            var.to_string(),
            VarInfo {
                ty: item_ty,
                from_observe: false,
                observe_key_literal: None,
                observe_commit_allowed: None,
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
                if let Some(false) = info.observe_commit_allowed {
                    let key_display = info
                        .observe_key_literal
                        .as_deref()
                        .unwrap_or("<dynamic-key>");
                    return Err(Diagnostic::new(
                        ErrorCode::TCommitForbiddenKey,
                        DiagPhase::Typecheck,
                        span,
                        format!("commit(...) is forbidden for observe-only key `{key_display}`"),
                    )
                    .with_hint("only ObserveAndCommit keys may be committed at compile-time"));
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
            Expr::Record { fields, .. } => {
                let mut record = BTreeMap::new();
                for (name, value) in fields {
                    if record.contains_key(name) {
                        return Err(self.type_error(
                            expr.span(),
                            format!("record literal has duplicate field `{name}`"),
                        ));
                    }
                    record.insert(name.clone(), self.infer_expr(value)?);
                }
                Ok(Type::Record(record))
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
                field,
                span: _,
            } => {
                let base_ty = self.infer_expr(base)?;
                match base_ty {
                    Type::Payload | Type::Unknown => Ok(Type::Unknown),
                    Type::Map(_, _) => Ok(Type::Unknown),
                    Type::Record(fields) => Ok(fields.get(field).cloned().unwrap_or(Type::Unknown)),
                    Type::Result4(_) => Ok(Type::Unknown),
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
            "len" => {
                if args.len() != 1 {
                    return Err(self.type_error(span, "len(...) expects exactly 1 argument"));
                }
                let arg_ty = self.infer_expr(&args[0])?;
                match arg_ty {
                    Type::List(_)
                    | Type::Map(_, _)
                    | Type::Record(_)
                    | Type::String
                    | Type::Payload => Ok(Type::Int),
                    other => Err(self.type_error(
                        span,
                        format!(
                            "len(...) expects list/map/string/payload, got {}",
                            other.as_str()
                        ),
                    )),
                }
            }
            "keys" => {
                if args.len() != 2 {
                    return Err(self.type_error(span, "keys(...) expects exactly 2 arguments"));
                }
                let map_ty = self.infer_expr(&args[0])?;
                let cap_ty = self.infer_expr(&args[1])?;
                if cap_ty != Type::Int {
                    return Err(self.type_error(span, "keys(...) cap argument must be int"));
                }
                match map_ty {
                    Type::Map(_, _) | Type::Record(_) | Type::Payload => {
                        Ok(Type::List(Box::new(Type::String)))
                    }
                    other => Err(self.type_error(
                        span,
                        format!("keys(...) expects map/payload, got {}", other.as_str()),
                    )),
                }
            }
            "merge" => {
                if args.len() != 3 {
                    return Err(self.type_error(span, "merge(...) expects exactly 3 arguments"));
                }
                let left_ty = self.infer_expr(&args[0])?;
                let right_ty = self.infer_expr(&args[1])?;
                let cap_ty = self.infer_expr(&args[2])?;
                if cap_ty != Type::Int {
                    return Err(self.type_error(span, "merge(...) cap argument must be int"));
                }
                if !matches!(left_ty, Type::Map(_, _) | Type::Record(_) | Type::Payload) {
                    return Err(self.type_error(
                        span,
                        format!(
                            "merge(...) left argument expects map/payload, got {}",
                            left_ty.as_str()
                        ),
                    ));
                }
                if !matches!(right_ty, Type::Map(_, _) | Type::Record(_) | Type::Payload) {
                    return Err(self.type_error(
                        span,
                        format!(
                            "merge(...) right argument expects map/payload, got {}",
                            right_ty.as_str()
                        ),
                    ));
                }
                Ok(Type::Map(Box::new(Type::String), Box::new(Type::Unknown)))
            }
            "concat" => {
                if args.is_empty() {
                    return Err(self.type_error(span, "concat(...) expects at least 1 argument"));
                }
                for arg in args {
                    let _ = self.infer_expr(arg)?;
                }
                Ok(Type::String)
            }
            "eq" | "ne" => {
                if args.len() != 2 {
                    return Err(
                        self.type_error(span, format!("{callee}(...) expects exactly 2 arguments"))
                    );
                }
                let _ = self.infer_expr(&args[0])?;
                let _ = self.infer_expr(&args[1])?;
                Ok(Type::Bool)
            }
            "lt" | "le" | "gt" | "ge" => {
                if args.len() != 2 {
                    return Err(
                        self.type_error(span, format!("{callee}(...) expects exactly 2 arguments"))
                    );
                }
                let left_ty = self.infer_expr(&args[0])?;
                let right_ty = self.infer_expr(&args[1])?;
                let numeric_pair = left_ty == Type::Int && right_ty == Type::Int;
                let string_pair = left_ty == Type::String && right_ty == Type::String;
                let unknown_pair = left_ty == Type::Unknown || right_ty == Type::Unknown;
                if !(numeric_pair || string_pair || unknown_pair) {
                    return Err(self.type_error(
                        span,
                        format!(
                            "{callee}(...) expects both arguments to be int or string, got {} and {}",
                            left_ty.as_str(),
                            right_ty.as_str()
                        ),
                    ));
                }
                Ok(Type::Bool)
            }
            "std.json.parse" => {
                if args.len() != 1 {
                    return Err(
                        self.type_error(span, "std.json.parse(...) expects exactly 1 argument")
                    );
                }
                let arg_ty = self.infer_expr(&args[0])?;
                if arg_ty != Type::String && arg_ty != Type::Unknown {
                    return Err(
                        self.type_error(span, "std.json.parse(...) argument must be string")
                    );
                }
                Ok(Type::Result4(Box::new(Type::Unknown)))
            }
            "std.hash.sha256" => {
                if args.len() != 1 {
                    return Err(
                        self.type_error(span, "std.hash.sha256(...) expects exactly 1 argument")
                    );
                }
                let arg_ty = self.infer_expr(&args[0])?;
                if arg_ty != Type::String && arg_ty != Type::Unknown {
                    return Err(
                        self.type_error(span, "std.hash.sha256(...) argument must be string")
                    );
                }
                Ok(Type::String)
            }
            "std.json.stringify" => {
                if args.len() != 1 {
                    return Err(
                        self.type_error(span, "std.json.stringify(...) expects exactly 1 argument")
                    );
                }
                let _ = self.infer_expr(&args[0])?;
                Ok(Type::String)
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

fn type_compatible(expected: &Type, got: &Type) -> bool {
    expected == got || *expected == Type::Unknown || *got == Type::Unknown
}

fn issue_code_to_error_code(code: SchemaIssueCode) -> ErrorCode {
    match code {
        SchemaIssueCode::MissingField => ErrorCode::TCtxMissingField,
        SchemaIssueCode::UnknownField => ErrorCode::TCtxUnknownField,
        SchemaIssueCode::TypeMismatch => ErrorCode::TCtxTypeMismatch,
        SchemaIssueCode::ConstraintViolation => ErrorCode::TCtxConstraintViolation,
    }
}

fn expr_literal_to_value(expr: &Expr) -> Option<Value> {
    match expr {
        Expr::Int { value, .. } => Some(Value::Int(*value)),
        Expr::Bool { value, .. } => Some(Value::Bool(*value)),
        Expr::String { value, .. } => Some(Value::String(value.clone())),
        Expr::List { items, .. } => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(expr_literal_to_value(item)?);
            }
            Some(Value::List(out))
        }
        Expr::Map { entries, .. } => {
            let mut out = BTreeMap::new();
            for (k, value) in entries {
                out.insert(k.clone(), expr_literal_to_value(value)?);
            }
            Some(Value::Map(out))
        }
        Expr::Record { fields, .. } => {
            let mut out = BTreeMap::new();
            for (k, value) in fields {
                out.insert(k.clone(), expr_literal_to_value(value)?);
            }
            Some(Value::Map(out))
        }
        _ => None,
    }
}

fn schema_type_compatible_with_type(schema: &SchemaType, ty: &Type) -> bool {
    match schema {
        SchemaType::Bool => matches!(ty, Type::Bool | Type::Unknown),
        SchemaType::Int => matches!(ty, Type::Int | Type::Unknown),
        SchemaType::String | SchemaType::Enum { .. } | SchemaType::Bytes => {
            matches!(ty, Type::String | Type::Unknown)
        }
        SchemaType::Null => matches!(ty, Type::Unit | Type::Unknown),
        SchemaType::List { elem, .. } => match ty {
            Type::List(inner) => schema_type_compatible_with_type(elem, inner),
            Type::Unknown => true,
            _ => false,
        },
        SchemaType::Map { value, .. } => match ty {
            Type::Map(_, map_value) => schema_type_compatible_with_type(value, map_value),
            Type::Record(fields) => fields
                .values()
                .all(|field_ty| schema_type_compatible_with_type(value, field_ty)),
            Type::Unknown => true,
            _ => false,
        },
        SchemaType::Record {
            fields: schema_fields,
            open_row,
        } => match ty {
            Type::Record(record_fields) => {
                for (name, spec) in schema_fields {
                    if spec.required {
                        let Some(record_ty) = record_fields.get(name) else {
                            return false;
                        };
                        if !schema_type_compatible_with_type(&spec.ty, record_ty) {
                            return false;
                        }
                    } else if let Some(record_ty) = record_fields.get(name) {
                        if !schema_type_compatible_with_type(&spec.ty, record_ty) {
                            return false;
                        }
                    }
                }
                if !open_row {
                    for name in record_fields.keys() {
                        if !schema_fields.contains_key(name) {
                            return false;
                        }
                    }
                }
                true
            }
            Type::Map(_, _) | Type::Unknown => true,
            _ => false,
        },
        SchemaType::Union { types } => types
            .iter()
            .any(|s| schema_type_compatible_with_type(s, ty)),
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}
