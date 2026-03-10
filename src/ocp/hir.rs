use std::collections::{BTreeMap, HashMap};

use sha2::{Digest, Sha256};

use crate::ocp::ast::{Expr, LetPattern, Program, Stmt};
use crate::ocp::diag::Diagnostic;
use crate::ocp::typecheck::typecheck_program;
use crate::ocp::types::Type;

pub const HIR_SCHEMA_VERSION: u32 = 1;
pub const HIR_HASHER_VERSION_V1: &str = "sha256-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HirPurity {
    Pure,
    CapabilityObserve,
    CapabilityCommit,
    Mixed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirProgram {
    pub statements: Vec<HirStmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirStmt {
    pub id: u64,
    pub ty: Type,
    pub purity: HirPurity,
    pub kind: HirStmtKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirStmtKind {
    ModuleDecl {
        path: Vec<String>,
    },
    ImportDecl {
        path: Vec<String>,
    },
    ExportDecl {
        name: String,
    },
    StructDecl {
        name: String,
        fields: Vec<String>,
    },
    EnumDecl {
        name: String,
        variants: Vec<String>,
    },
    FnDef {
        name: String,
        params: Vec<String>,
        body: Vec<HirStmt>,
    },
    Let {
        pattern: HirLetPattern,
        value: HirExpr,
    },
    Return {
        value: HirExpr,
    },
    TryLet {
        name: String,
        value: HirExpr,
        else_expr: HirExpr,
        else_returns: bool,
    },
    Guard {
        value: HirExpr,
    },
    Repeat {
        count: HirExpr,
        body: Vec<HirStmt>,
    },
    ForEachCap {
        var: String,
        iter: HirExpr,
        cap: HirExpr,
        body: Vec<HirStmt>,
    },
    ForRange {
        var: String,
        start: HirExpr,
        end: HirExpr,
        body: Vec<HirStmt>,
    },
    Observe {
        key: HirExpr,
        tier: HirExpr,
        ctx: HirExpr,
        budget: HirExpr,
        bind: String,
    },
    Commit {
        value: HirExpr,
    },
    Condition {
        value: HirExpr,
    },
    Entangle {
        left: String,
        right: String,
        constraint: HirExpr,
    },
    Match {
        value: HirExpr,
        ok_arm: Vec<HirStmt>,
        degraded_arm: Vec<HirStmt>,
        insufficient_arm: Vec<HirStmt>,
        deferred_arm: Vec<HirStmt>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirLetPattern {
    Ident(String),
    Record(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirExpr {
    pub id: u64,
    pub ty: Type,
    pub purity: HirPurity,
    pub kind: HirExprKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirExprKind {
    Int(i64),
    Bool(bool),
    String(String),
    Ident(String),
    Call { callee: String, args: Vec<HirExpr> },
    List(Vec<HirExpr>),
    Map(Vec<(String, HirExpr)>),
    Record(Vec<(String, HirExpr)>),
    Try(Box<HirExpr>),
    FieldAccess { base: Box<HirExpr>, field: String },
}

pub fn build_hir(program: &Program) -> Result<HirProgram, Diagnostic> {
    typecheck_program(program)?;
    let lowerer = HirLowerer::new(program);
    Ok(lowerer.lower_program(program))
}

pub fn build_hir_and_hash(program: &Program) -> Result<(HirProgram, String), Diagnostic> {
    let hir = build_hir(program)?;
    let hash = ir_hash_sha256_v1(&hir);
    Ok((hir, hash))
}

pub fn canonical_hir_bytes(program: &HirProgram) -> Vec<u8> {
    let mut out = String::new();
    out.push_str("hir_schema_version=");
    out.push_str(&HIR_SCHEMA_VERSION.to_string());
    out.push('\n');
    for stmt in &program.statements {
        write_stmt(stmt, &mut out);
        out.push('\n');
    }
    out.into_bytes()
}

pub fn ir_hash_sha256_v1(program: &HirProgram) -> String {
    let mut hasher = Sha256::new();
    hasher.update(canonical_hir_bytes(program));
    let digest = hasher.finalize();
    bytes_to_hex(digest.as_slice())
}

struct HirLowerer {
    next_id: u64,
    vars: HashMap<String, Type>,
    fn_returns: HashMap<String, Type>,
}

impl HirLowerer {
    fn new(program: &Program) -> Self {
        let mut fn_returns = HashMap::new();
        for stmt in &program.statements {
            if let Stmt::FnDef { name, .. } = stmt {
                fn_returns.insert(name.clone(), Type::Unknown);
            }
        }
        Self {
            next_id: 0,
            vars: HashMap::new(),
            fn_returns,
        }
    }

    fn lower_program(mut self, program: &Program) -> HirProgram {
        let statements = program
            .statements
            .iter()
            .map(|stmt| self.lower_stmt(stmt))
            .collect();
        HirProgram { statements }
    }

    fn lower_stmt(&mut self, stmt: &Stmt) -> HirStmt {
        let id = self.alloc_id();
        match stmt {
            Stmt::ModuleDecl { path, .. } => HirStmt {
                id,
                ty: Type::Unit,
                purity: HirPurity::Pure,
                kind: HirStmtKind::ModuleDecl { path: path.clone() },
            },
            Stmt::ImportDecl { path, .. } => HirStmt {
                id,
                ty: Type::Unit,
                purity: HirPurity::Pure,
                kind: HirStmtKind::ImportDecl { path: path.clone() },
            },
            Stmt::ExportDecl { name, .. } => HirStmt {
                id,
                ty: Type::Unit,
                purity: HirPurity::Pure,
                kind: HirStmtKind::ExportDecl { name: name.clone() },
            },
            Stmt::StructDecl { name, fields, .. } => {
                let mut sorted_fields = fields.clone();
                sorted_fields.sort();
                HirStmt {
                    id,
                    ty: Type::Unit,
                    purity: HirPurity::Pure,
                    kind: HirStmtKind::StructDecl {
                        name: name.clone(),
                        fields: sorted_fields,
                    },
                }
            }
            Stmt::EnumDecl { name, variants, .. } => {
                let mut sorted_variants = variants.clone();
                sorted_variants.sort();
                HirStmt {
                    id,
                    ty: Type::Unit,
                    purity: HirPurity::Pure,
                    kind: HirStmtKind::EnumDecl {
                        name: name.clone(),
                        variants: sorted_variants,
                    },
                }
            }
            Stmt::FnDef {
                name, params, body, ..
            } => {
                let snapshot = self.vars.clone();
                for param in params {
                    self.vars.insert(param.clone(), Type::Unknown);
                }
                let lowered_body = body.iter().map(|s| self.lower_stmt(s)).collect::<Vec<_>>();
                self.fn_returns
                    .insert(name.clone(), infer_function_return_type(&lowered_body));
                self.vars = snapshot;
                HirStmt {
                    id,
                    ty: Type::Unit,
                    purity: merge_purities(lowered_body.iter().map(|s| s.purity)),
                    kind: HirStmtKind::FnDef {
                        name: name.clone(),
                        params: params.clone(),
                        body: lowered_body,
                    },
                }
            }
            Stmt::Let { pattern, value, .. } => {
                let lowered_value = self.lower_expr(value);
                let lowered_pattern = canonical_pattern(pattern);
                self.bind_pattern(&lowered_pattern, &lowered_value.ty);
                HirStmt {
                    id,
                    ty: Type::Unit,
                    purity: lowered_value.purity,
                    kind: HirStmtKind::Let {
                        pattern: lowered_pattern,
                        value: lowered_value,
                    },
                }
            }
            Stmt::Return { value, .. } => {
                let lowered_value = self.lower_expr(value);
                HirStmt {
                    id,
                    ty: lowered_value.ty.clone(),
                    purity: lowered_value.purity,
                    kind: HirStmtKind::Return {
                        value: lowered_value,
                    },
                }
            }
            Stmt::TryLet {
                name,
                value,
                else_expr,
                else_returns,
                ..
            } => {
                let lowered_value = self.lower_expr(value);
                let lowered_else = self.lower_expr(else_expr);
                let bound_ty = match &lowered_value.ty {
                    Type::Result4(inner) => (*inner.as_ref()).clone(),
                    _ => Type::Unknown,
                };
                self.vars.insert(name.clone(), bound_ty);
                HirStmt {
                    id,
                    ty: Type::Unit,
                    purity: merge_purities([lowered_value.purity, lowered_else.purity]),
                    kind: HirStmtKind::TryLet {
                        name: name.clone(),
                        value: lowered_value,
                        else_expr: lowered_else,
                        else_returns: *else_returns,
                    },
                }
            }
            Stmt::Guard { value, .. } => {
                let lowered_value = self.lower_expr(value);
                HirStmt {
                    id,
                    ty: Type::Unit,
                    purity: lowered_value.purity,
                    kind: HirStmtKind::Guard {
                        value: lowered_value,
                    },
                }
            }
            Stmt::Repeat { count, body, .. } => {
                let lowered_count = self.lower_expr(count);
                let lowered_body = self.lower_block(body);
                HirStmt {
                    id,
                    ty: Type::Unit,
                    purity: merge_purities(
                        [lowered_count.purity]
                            .into_iter()
                            .chain(lowered_body.iter().map(|s| s.purity)),
                    ),
                    kind: HirStmtKind::Repeat {
                        count: lowered_count,
                        body: lowered_body,
                    },
                }
            }
            Stmt::ForEachCap {
                var,
                iter,
                cap,
                body,
                ..
            } => {
                let lowered_iter = self.lower_expr(iter);
                let lowered_cap = self.lower_expr(cap);
                let item_ty = match &lowered_iter.ty {
                    Type::List(inner) => (*inner.as_ref()).clone(),
                    _ => Type::Unknown,
                };
                let snapshot = self.vars.clone();
                self.vars.insert(var.clone(), item_ty);
                let lowered_body = body.iter().map(|s| self.lower_stmt(s)).collect::<Vec<_>>();
                self.vars = snapshot;
                HirStmt {
                    id,
                    ty: Type::Unit,
                    purity: merge_purities(
                        [lowered_iter.purity, lowered_cap.purity]
                            .into_iter()
                            .chain(lowered_body.iter().map(|s| s.purity)),
                    ),
                    kind: HirStmtKind::ForEachCap {
                        var: var.clone(),
                        iter: lowered_iter,
                        cap: lowered_cap,
                        body: lowered_body,
                    },
                }
            }
            Stmt::ForRange {
                var,
                start,
                end,
                body,
                ..
            } => {
                let lowered_start = self.lower_expr(start);
                let lowered_end = self.lower_expr(end);
                let snapshot = self.vars.clone();
                self.vars.insert(var.clone(), Type::Int);
                let lowered_body = body.iter().map(|s| self.lower_stmt(s)).collect::<Vec<_>>();
                self.vars = snapshot;
                HirStmt {
                    id,
                    ty: Type::Unit,
                    purity: merge_purities(
                        [lowered_start.purity, lowered_end.purity]
                            .into_iter()
                            .chain(lowered_body.iter().map(|s| s.purity)),
                    ),
                    kind: HirStmtKind::ForRange {
                        var: var.clone(),
                        start: lowered_start,
                        end: lowered_end,
                        body: lowered_body,
                    },
                }
            }
            Stmt::Observe {
                key,
                tier,
                ctx,
                budget,
                bind,
                ..
            } => {
                let lowered_key = self.lower_expr(key);
                let lowered_tier = self.lower_expr(tier);
                let lowered_ctx = self.lower_expr(ctx);
                let lowered_budget = self.lower_expr(budget);
                let bind_ty = Type::result4_payload();
                self.vars.insert(bind.clone(), bind_ty.clone());
                HirStmt {
                    id,
                    ty: bind_ty,
                    purity: merge_purities([
                        HirPurity::CapabilityObserve,
                        lowered_key.purity,
                        lowered_tier.purity,
                        lowered_ctx.purity,
                        lowered_budget.purity,
                    ]),
                    kind: HirStmtKind::Observe {
                        key: lowered_key,
                        tier: lowered_tier,
                        ctx: lowered_ctx,
                        budget: lowered_budget,
                        bind: bind.clone(),
                    },
                }
            }
            Stmt::Commit { value, .. } => {
                let lowered_value = self.lower_expr(value);
                HirStmt {
                    id,
                    ty: Type::Unit,
                    purity: merge_purities([HirPurity::CapabilityCommit, lowered_value.purity]),
                    kind: HirStmtKind::Commit {
                        value: lowered_value,
                    },
                }
            }
            Stmt::Condition { value, .. } => {
                let lowered_value = self.lower_expr(value);
                HirStmt {
                    id,
                    ty: Type::Unit,
                    purity: lowered_value.purity,
                    kind: HirStmtKind::Condition {
                        value: lowered_value,
                    },
                }
            }
            Stmt::Entangle {
                left,
                right,
                constraint,
                ..
            } => {
                let lowered_constraint = self.lower_expr(constraint);
                HirStmt {
                    id,
                    ty: Type::Unit,
                    purity: lowered_constraint.purity,
                    kind: HirStmtKind::Entangle {
                        left: left.clone(),
                        right: right.clone(),
                        constraint: lowered_constraint,
                    },
                }
            }
            Stmt::Match(m) => {
                let lowered_value = self.lower_expr(&m.value);
                let ok_arm = self.lower_block(&m.ok_arm);
                let degraded_arm = self.lower_block(&m.degraded_arm);
                let insufficient_arm = self.lower_block(&m.insufficient_arm);
                let deferred_arm = self.lower_block(&m.deferred_arm);
                HirStmt {
                    id,
                    ty: Type::Unit,
                    purity: merge_purities(
                        [lowered_value.purity]
                            .into_iter()
                            .chain(ok_arm.iter().map(|s| s.purity))
                            .chain(degraded_arm.iter().map(|s| s.purity))
                            .chain(insufficient_arm.iter().map(|s| s.purity))
                            .chain(deferred_arm.iter().map(|s| s.purity)),
                    ),
                    kind: HirStmtKind::Match {
                        value: lowered_value,
                        ok_arm,
                        degraded_arm,
                        insufficient_arm,
                        deferred_arm,
                    },
                }
            }
        }
    }

    fn lower_block(&mut self, stmts: &[Stmt]) -> Vec<HirStmt> {
        let snapshot = self.vars.clone();
        let lowered = stmts.iter().map(|stmt| self.lower_stmt(stmt)).collect();
        self.vars = snapshot;
        lowered
    }

    fn lower_expr(&mut self, expr: &Expr) -> HirExpr {
        let id = self.alloc_id();
        match expr {
            Expr::Int { value, .. } => HirExpr {
                id,
                ty: Type::Int,
                purity: HirPurity::Pure,
                kind: HirExprKind::Int(*value),
            },
            Expr::Bool { value, .. } => HirExpr {
                id,
                ty: Type::Bool,
                purity: HirPurity::Pure,
                kind: HirExprKind::Bool(*value),
            },
            Expr::String { value, .. } => HirExpr {
                id,
                ty: Type::String,
                purity: HirPurity::Pure,
                kind: HirExprKind::String(value.clone()),
            },
            Expr::Ident { name, .. } => HirExpr {
                id,
                ty: self.vars.get(name).cloned().unwrap_or(Type::Unknown),
                purity: HirPurity::Pure,
                kind: HirExprKind::Ident(name.clone()),
            },
            Expr::Call { callee, args, .. } => {
                let lowered_args = args
                    .iter()
                    .map(|arg| self.lower_expr(arg))
                    .collect::<Vec<_>>();
                let arg_types = lowered_args
                    .iter()
                    .map(|arg| arg.ty.clone())
                    .collect::<Vec<_>>();
                HirExpr {
                    id,
                    ty: self.infer_call_type(callee, &arg_types),
                    purity: merge_purities(lowered_args.iter().map(|arg| arg.purity)),
                    kind: HirExprKind::Call {
                        callee: callee.clone(),
                        args: lowered_args,
                    },
                }
            }
            Expr::List { items, .. } => {
                let lowered_items = items
                    .iter()
                    .map(|item| self.lower_expr(item))
                    .collect::<Vec<_>>();
                let item_ty = merge_types(lowered_items.iter().map(|item| item.ty.clone()));
                HirExpr {
                    id,
                    ty: Type::List(Box::new(item_ty)),
                    purity: merge_purities(lowered_items.iter().map(|item| item.purity)),
                    kind: HirExprKind::List(lowered_items),
                }
            }
            Expr::Map { entries, .. } => {
                let mut sorted = entries.iter().collect::<Vec<_>>();
                sorted.sort_by(|(left, _), (right, _)| left.cmp(right));
                let mut lowered_entries = Vec::with_capacity(sorted.len());
                for (key, value) in sorted {
                    lowered_entries.push((key.clone(), self.lower_expr(value)));
                }
                let value_ty =
                    merge_types(lowered_entries.iter().map(|(_, value)| value.ty.clone()));
                HirExpr {
                    id,
                    ty: Type::Map(Box::new(Type::String), Box::new(value_ty)),
                    purity: merge_purities(lowered_entries.iter().map(|(_, value)| value.purity)),
                    kind: HirExprKind::Map(lowered_entries),
                }
            }
            Expr::Record { fields, .. } => {
                let mut sorted = fields.iter().collect::<Vec<_>>();
                sorted.sort_by(|(left, _), (right, _)| left.cmp(right));
                let mut lowered_fields = Vec::with_capacity(sorted.len());
                let mut record_ty = BTreeMap::new();
                for (key, value) in sorted {
                    let lowered_value = self.lower_expr(value);
                    record_ty.insert(key.clone(), lowered_value.ty.clone());
                    lowered_fields.push((key.clone(), lowered_value));
                }
                HirExpr {
                    id,
                    ty: Type::Record(record_ty),
                    purity: merge_purities(lowered_fields.iter().map(|(_, value)| value.purity)),
                    kind: HirExprKind::Record(lowered_fields),
                }
            }
            Expr::Try { value, .. } => {
                let lowered_value = self.lower_expr(value);
                let ty = match &lowered_value.ty {
                    Type::Result4(inner) => (*inner.as_ref()).clone(),
                    _ => Type::Unknown,
                };
                HirExpr {
                    id,
                    ty,
                    purity: lowered_value.purity,
                    kind: HirExprKind::Try(Box::new(lowered_value)),
                }
            }
            Expr::FieldAccess { base, field, .. } => {
                let lowered_base = self.lower_expr(base);
                let ty = match &lowered_base.ty {
                    Type::Record(fields) => fields.get(field).cloned().unwrap_or(Type::Unknown),
                    Type::Map(_, value_ty) => (*value_ty.as_ref()).clone(),
                    _ => Type::Unknown,
                };
                HirExpr {
                    id,
                    ty,
                    purity: lowered_base.purity,
                    kind: HirExprKind::FieldAccess {
                        base: Box::new(lowered_base),
                        field: field.clone(),
                    },
                }
            }
        }
    }

    fn bind_pattern(&mut self, pattern: &HirLetPattern, value_ty: &Type) {
        match pattern {
            HirLetPattern::Ident(name) => {
                self.vars.insert(name.clone(), value_ty.clone());
            }
            HirLetPattern::Record(fields) => {
                for field in fields {
                    let ty = match value_ty {
                        Type::Record(values) => values.get(field).cloned().unwrap_or(Type::Unknown),
                        Type::Map(_, value_ty) => (*value_ty.as_ref()).clone(),
                        _ => Type::Unknown,
                    };
                    self.vars.insert(field.clone(), ty);
                }
            }
        }
    }

    fn infer_call_type(&self, callee: &str, _args: &[Type]) -> Type {
        match callee {
            "budget" => Type::Budget,
            "len" => Type::Int,
            "keys" => Type::List(Box::new(Type::String)),
            "merge" => Type::Map(Box::new(Type::String), Box::new(Type::Unknown)),
            "std.json.parse" => Type::Result4(Box::new(Type::Unknown)),
            "std.hash.sha256" => Type::String,
            "std.json.stringify" => Type::String,
            "ctx" => Type::Ctx,
            "payload" => Type::Payload,
            _ => self
                .fn_returns
                .get(callee)
                .cloned()
                .unwrap_or(Type::Unknown),
        }
    }

    fn alloc_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

fn infer_function_return_type(body: &[HirStmt]) -> Type {
    let mut return_types = Vec::new();
    collect_return_types(body, &mut return_types);
    merge_types(return_types)
}

fn collect_return_types(stmts: &[HirStmt], output: &mut Vec<Type>) {
    for stmt in stmts {
        match &stmt.kind {
            HirStmtKind::Return { value } => output.push(value.ty.clone()),
            HirStmtKind::FnDef { body, .. }
            | HirStmtKind::Repeat { body, .. }
            | HirStmtKind::ForEachCap { body, .. }
            | HirStmtKind::ForRange { body, .. } => collect_return_types(body, output),
            HirStmtKind::Match {
                ok_arm,
                degraded_arm,
                insufficient_arm,
                deferred_arm,
                ..
            } => {
                collect_return_types(ok_arm, output);
                collect_return_types(degraded_arm, output);
                collect_return_types(insufficient_arm, output);
                collect_return_types(deferred_arm, output);
            }
            _ => {}
        }
    }
}

fn canonical_pattern(pattern: &LetPattern) -> HirLetPattern {
    match pattern {
        LetPattern::Ident(name) => HirLetPattern::Ident(name.clone()),
        LetPattern::Record(fields) => {
            let mut sorted = fields.clone();
            sorted.sort();
            HirLetPattern::Record(sorted)
        }
    }
}

fn merge_types<I>(types: I) -> Type
where
    I: IntoIterator<Item = Type>,
{
    let mut iter = types.into_iter();
    let Some(mut acc) = iter.next() else {
        return Type::Unknown;
    };
    for ty in iter {
        acc = merge_two_types(acc, ty);
    }
    acc
}

fn merge_two_types(left: Type, right: Type) -> Type {
    if left == right {
        return left;
    }
    if matches!(left, Type::Unknown) {
        return right;
    }
    if matches!(right, Type::Unknown) {
        return left;
    }
    Type::Unknown
}

fn merge_purities<I>(purities: I) -> HirPurity
where
    I: IntoIterator<Item = HirPurity>,
{
    let mut acc = HirPurity::Pure;
    for purity in purities {
        acc = merge_two_purity(acc, purity);
        if matches!(acc, HirPurity::Mixed) {
            break;
        }
    }
    acc
}

fn merge_two_purity(left: HirPurity, right: HirPurity) -> HirPurity {
    match (left, right) {
        (HirPurity::Mixed, _) | (_, HirPurity::Mixed) => HirPurity::Mixed,
        (HirPurity::Pure, other) | (other, HirPurity::Pure) => other,
        (HirPurity::CapabilityObserve, HirPurity::CapabilityObserve) => {
            HirPurity::CapabilityObserve
        }
        (HirPurity::CapabilityCommit, HirPurity::CapabilityCommit) => HirPurity::CapabilityCommit,
        _ => HirPurity::Mixed,
    }
}

fn write_stmt(stmt: &HirStmt, out: &mut String) {
    out.push_str("S(");
    out.push_str(&stmt.id.to_string());
    out.push('|');
    write_type(&stmt.ty, out);
    out.push('|');
    write_purity(stmt.purity, out);
    out.push('|');
    match &stmt.kind {
        HirStmtKind::ModuleDecl { path } => {
            out.push_str("module:");
            write_string_list(path, out);
        }
        HirStmtKind::ImportDecl { path } => {
            out.push_str("import:");
            write_string_list(path, out);
        }
        HirStmtKind::ExportDecl { name } => {
            out.push_str("export:");
            write_string(name, out);
        }
        HirStmtKind::StructDecl { name, fields } => {
            out.push_str("struct:");
            write_string(name, out);
            out.push('=');
            write_string_list(fields, out);
        }
        HirStmtKind::EnumDecl { name, variants } => {
            out.push_str("enum:");
            write_string(name, out);
            out.push('=');
            write_string_list(variants, out);
        }
        HirStmtKind::FnDef { name, params, body } => {
            out.push_str("fn:");
            write_string(name, out);
            out.push('=');
            write_string_list(params, out);
            out.push('=');
            write_stmt_list(body, out);
        }
        HirStmtKind::Let { pattern, value } => {
            out.push_str("let:");
            write_pattern(pattern, out);
            out.push('=');
            write_expr(value, out);
        }
        HirStmtKind::Return { value } => {
            out.push_str("return:");
            write_expr(value, out);
        }
        HirStmtKind::TryLet {
            name,
            value,
            else_expr,
            else_returns,
        } => {
            out.push_str("trylet:");
            write_string(name, out);
            out.push('=');
            write_expr(value, out);
            out.push('=');
            write_expr(else_expr, out);
            out.push('=');
            out.push_str(if *else_returns { "ret" } else { "yield" });
        }
        HirStmtKind::Guard { value } => {
            out.push_str("guard:");
            write_expr(value, out);
        }
        HirStmtKind::Repeat { count, body } => {
            out.push_str("repeat:");
            write_expr(count, out);
            out.push('=');
            write_stmt_list(body, out);
        }
        HirStmtKind::ForEachCap {
            var,
            iter,
            cap,
            body,
        } => {
            out.push_str("foreachcap:");
            write_string(var, out);
            out.push('=');
            write_expr(iter, out);
            out.push('=');
            write_expr(cap, out);
            out.push('=');
            write_stmt_list(body, out);
        }
        HirStmtKind::ForRange {
            var,
            start,
            end,
            body,
        } => {
            out.push_str("forrange:");
            write_string(var, out);
            out.push('=');
            write_expr(start, out);
            out.push('=');
            write_expr(end, out);
            out.push('=');
            write_stmt_list(body, out);
        }
        HirStmtKind::Observe {
            key,
            tier,
            ctx,
            budget,
            bind,
        } => {
            out.push_str("observe:");
            write_string(bind, out);
            out.push('=');
            write_expr(key, out);
            out.push('=');
            write_expr(tier, out);
            out.push('=');
            write_expr(ctx, out);
            out.push('=');
            write_expr(budget, out);
        }
        HirStmtKind::Commit { value } => {
            out.push_str("commit:");
            write_expr(value, out);
        }
        HirStmtKind::Condition { value } => {
            out.push_str("condition:");
            write_expr(value, out);
        }
        HirStmtKind::Entangle {
            left,
            right,
            constraint,
        } => {
            out.push_str("entangle:");
            write_string(left, out);
            out.push('=');
            write_string(right, out);
            out.push('=');
            write_expr(constraint, out);
        }
        HirStmtKind::Match {
            value,
            ok_arm,
            degraded_arm,
            insufficient_arm,
            deferred_arm,
        } => {
            out.push_str("match:");
            write_expr(value, out);
            out.push('=');
            write_stmt_list(ok_arm, out);
            out.push('=');
            write_stmt_list(degraded_arm, out);
            out.push('=');
            write_stmt_list(insufficient_arm, out);
            out.push('=');
            write_stmt_list(deferred_arm, out);
        }
    }
    out.push(')');
}

fn write_stmt_list(stmts: &[HirStmt], out: &mut String) {
    out.push('[');
    for (idx, stmt) in stmts.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        write_stmt(stmt, out);
    }
    out.push(']');
}

fn write_pattern(pattern: &HirLetPattern, out: &mut String) {
    match pattern {
        HirLetPattern::Ident(name) => {
            out.push_str("id:");
            write_string(name, out);
        }
        HirLetPattern::Record(fields) => {
            out.push_str("record:");
            write_string_list(fields, out);
        }
    }
}

fn write_expr(expr: &HirExpr, out: &mut String) {
    out.push_str("E(");
    out.push_str(&expr.id.to_string());
    out.push('|');
    write_type(&expr.ty, out);
    out.push('|');
    write_purity(expr.purity, out);
    out.push('|');
    match &expr.kind {
        HirExprKind::Int(value) => {
            out.push_str("int:");
            out.push_str(&value.to_string());
        }
        HirExprKind::Bool(value) => {
            out.push_str("bool:");
            out.push_str(if *value { "1" } else { "0" });
        }
        HirExprKind::String(value) => {
            out.push_str("str:");
            write_string(value, out);
        }
        HirExprKind::Ident(name) => {
            out.push_str("ident:");
            write_string(name, out);
        }
        HirExprKind::Call { callee, args } => {
            out.push_str("call:");
            write_string(callee, out);
            out.push('=');
            write_expr_list(args, out);
        }
        HirExprKind::List(items) => {
            out.push_str("list:");
            write_expr_list(items, out);
        }
        HirExprKind::Map(entries) => {
            out.push_str("map:{");
            for (idx, (key, value)) in entries.iter().enumerate() {
                if idx > 0 {
                    out.push(',');
                }
                write_string(key, out);
                out.push('=');
                write_expr(value, out);
            }
            out.push('}');
        }
        HirExprKind::Record(fields) => {
            out.push_str("record:{");
            for (idx, (key, value)) in fields.iter().enumerate() {
                if idx > 0 {
                    out.push(',');
                }
                write_string(key, out);
                out.push('=');
                write_expr(value, out);
            }
            out.push('}');
        }
        HirExprKind::Try(value) => {
            out.push_str("try:");
            write_expr(value, out);
        }
        HirExprKind::FieldAccess { base, field } => {
            out.push_str("field:");
            write_expr(base, out);
            out.push('=');
            write_string(field, out);
        }
    }
    out.push(')');
}

fn write_expr_list(items: &[HirExpr], out: &mut String) {
    out.push('[');
    for (idx, item) in items.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        write_expr(item, out);
    }
    out.push(']');
}

fn write_string_list(items: &[String], out: &mut String) {
    out.push('[');
    for (idx, item) in items.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        write_string(item, out);
    }
    out.push(']');
}

fn write_string(value: &str, out: &mut String) {
    out.push('x');
    out.push_str(&bytes_to_hex(value.as_bytes()));
}

fn write_purity(purity: HirPurity, out: &mut String) {
    match purity {
        HirPurity::Pure => out.push_str("pure"),
        HirPurity::CapabilityObserve => out.push_str("observe"),
        HirPurity::CapabilityCommit => out.push_str("commit"),
        HirPurity::Mixed => out.push_str("mixed"),
    }
}

fn write_type(ty: &Type, out: &mut String) {
    match ty {
        Type::Int => out.push_str("int"),
        Type::Bool => out.push_str("bool"),
        Type::String => out.push_str("string"),
        Type::List(inner) => {
            out.push_str("list<");
            write_type(inner, out);
            out.push('>');
        }
        Type::Map(key, value) => {
            out.push_str("map<");
            write_type(key, out);
            out.push(',');
            write_type(value, out);
            out.push('>');
        }
        Type::Record(fields) => {
            out.push_str("record{");
            let mut first = true;
            for (name, field_ty) in fields {
                if !first {
                    out.push(',');
                }
                first = false;
                write_string(name, out);
                out.push(':');
                write_type(field_ty, out);
            }
            out.push('}');
        }
        Type::Budget => out.push_str("budget"),
        Type::Ctx => out.push_str("ctx"),
        Type::Payload => out.push_str("payload"),
        Type::Result4(inner) => {
            out.push_str("result4<");
            write_type(inner, out);
            out.push('>');
        }
        Type::Unit => out.push_str("unit"),
        Type::Unknown => out.push_str("unknown"),
    }
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}
