use std::collections::{BTreeMap, HashMap, HashSet};

use crate::ocp_ocl::ast::{Expr, MatchStmt, Program, Stmt};
use crate::ocp_ocl::audit::{TraceEvent, TraceLog};
use crate::ocp_ocl::budget::{BudgetMeter, ExecConfig};
use crate::ocp_ocl::diag::{DiagPhase, Diagnostic, ErrorCode, ReasonCode};
use crate::ocp_ocl::registry::CapabilityRegistry;
use crate::ocp_ocl::result_kind::{Result4, ResultKind};
use crate::ocp_ocl::value::Value;
use crate::ocp_ocl::Span;

pub fn execute_program(program: &Program, config: ExecConfig) -> Result<ExecOutput, Diagnostic> {
    Executor::new(config).run(program)
}

#[derive(Debug, Clone)]
pub struct ExecOutput {
    pub env: HashMap<String, Value>,
    pub steps: u32,
    pub commits: Vec<CommitEvent>,
    pub trace: TraceLog,
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitEvent {
    pub origin_id: u64,
    pub key: String,
    pub kind: ResultKind,
}

#[derive(Debug, Clone)]
struct ObservationMeta {
    key: String,
}

const CONDITION_TIME_BUDGET_NS: u64 = 20_000;
const CONDITION_MAX_STEPS: usize = 64;
const CONDITION_MAX_CONSTRAINTS: usize = 256;
const ENTANGLE_MAX_EDGES_PER_SESSION: usize = 128;
const ENTANGLE_MAX_DEGREE_PER_BINDING: usize = 16;
const ENTANGLE_PROPAGATION_BUDGET_NS: u64 = 20_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConditionOutcome {
    Pass,
    Fail,
    Deferred(ReasonCode),
    Insufficient(ReasonCode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ConditionBudgetStats {
    steps: usize,
    constraints: usize,
}

#[derive(Debug, Clone, Default)]
struct ConstraintSession {
    edge_count: usize,
    degree_by_binding: HashMap<String, usize>,
}

#[derive(Debug, Clone)]
struct FunctionDefRuntime {
    params: Vec<String>,
    body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
enum Flow {
    Continue,
    Return(Value),
}

pub struct Executor {
    env: HashMap<String, Value>,
    meter: BudgetMeter,
    next_origin_id: u64,
    registry: CapabilityRegistry,
    observations: HashMap<u64, ObservationMeta>,
    commits: Vec<CommitEvent>,
    trace: TraceLog,
    constraints: ConstraintSession,
    functions: HashMap<String, FunctionDefRuntime>,
}

impl Executor {
    pub fn new(config: ExecConfig) -> Self {
        Self {
            env: HashMap::new(),
            meter: BudgetMeter::new(config),
            next_origin_id: 1,
            registry: CapabilityRegistry::default(),
            observations: HashMap::new(),
            commits: Vec::new(),
            trace: TraceLog::default(),
            constraints: ConstraintSession::default(),
            functions: HashMap::new(),
        }
    }

    pub fn with_registry(config: ExecConfig, registry: CapabilityRegistry) -> Self {
        Self {
            env: HashMap::new(),
            meter: BudgetMeter::new(config),
            next_origin_id: 1,
            registry,
            observations: HashMap::new(),
            commits: Vec::new(),
            trace: TraceLog::default(),
            constraints: ConstraintSession::default(),
            functions: HashMap::new(),
        }
    }

    pub fn run(mut self, program: &Program) -> Result<ExecOutput, Diagnostic> {
        for stmt in &program.statements {
            if let Stmt::FnDef {
                name, params, body, ..
            } = stmt
            {
                self.functions.insert(
                    name.clone(),
                    FunctionDefRuntime {
                        params: params.clone(),
                        body: body.clone(),
                    },
                );
            }
        }

        for stmt in &program.statements {
            match self.exec_stmt_inner(stmt, false)? {
                Flow::Continue => {}
                Flow::Return(_) => {
                    return Err(Diagnostic::new(
                        ErrorCode::XCommitForbidden,
                        DiagPhase::Exec,
                        stmt_span(stmt),
                        "return is only allowed inside function body",
                    ));
                }
            }
        }
        self.trace.push(TraceEvent::ProgramEnd {
            steps: self.meter.steps(),
        });
        let signature = self.trace.signature_hex();
        Ok(ExecOutput {
            env: self.env,
            steps: self.meter.steps(),
            commits: self.commits,
            trace: self.trace,
            signature,
        })
    }

    fn exec_block(&mut self, block: &[Stmt], in_function: bool) -> Result<Flow, Diagnostic> {
        for stmt in block {
            match self.exec_stmt_inner(stmt, in_function)? {
                Flow::Continue => {}
                Flow::Return(v) => return Ok(Flow::Return(v)),
            }
        }
        Ok(Flow::Continue)
    }

    fn exec_stmt_inner(&mut self, stmt: &Stmt, in_function: bool) -> Result<Flow, Diagnostic> {
        self.tick(stmt_span(stmt))?;
        match stmt {
            Stmt::ModuleDecl { .. }
            | Stmt::ImportDecl { .. }
            | Stmt::ExportDecl { .. }
            | Stmt::StructDecl { .. }
            | Stmt::EnumDecl { .. }
            | Stmt::FnDef { .. } => Ok(Flow::Continue),
            Stmt::Let { name, value, .. } => {
                let v = self.eval_expr(value)?;
                self.env.insert(name.clone(), v);
                Ok(Flow::Continue)
            }
            Stmt::Return { value, span } => {
                if !in_function {
                    return Err(Diagnostic::new(
                        ErrorCode::XCommitForbidden,
                        DiagPhase::Exec,
                        *span,
                        "return is only allowed inside function body",
                    ));
                }
                let v = self.eval_expr(value)?;
                Ok(Flow::Return(v))
            }
            Stmt::ForRange {
                var,
                start,
                end,
                body,
                span,
            } => self.exec_for_range(var, start, end, body, *span, in_function),
            Stmt::Observe {
                key,
                tier,
                ctx,
                budget,
                bind,
                ..
            } => {
                let key_value = self.eval_expr(key)?;
                let tier_value = self.eval_expr(tier)?;
                let ctx_value = self.eval_expr(ctx)?;
                let budget_value = self.eval_expr(budget)?;

                let key_lit = as_string_runtime(&key_value).ok_or_else(|| {
                    Diagnostic::new(
                        ErrorCode::RCapabilityDenied,
                        DiagPhase::Exec,
                        key.span(),
                        "observe key must evaluate to string",
                    )
                })?;
                let tier_lit = as_string_runtime(&tier_value).ok_or_else(|| {
                    Diagnostic::new(
                        ErrorCode::RCapabilityDenied,
                        DiagPhase::Exec,
                        tier.span(),
                        "observe tier must evaluate to string",
                    )
                })?;
                let ctx_lit = as_ctx_runtime(&ctx_value).ok_or_else(|| {
                    Diagnostic::new(
                        ErrorCode::RCapabilityDenied,
                        DiagPhase::Exec,
                        ctx.span(),
                        "observe ctx must evaluate to ctx(...) value",
                    )
                })?;
                let budget_units = as_budget_runtime(&budget_value).ok_or_else(|| {
                    Diagnostic::new(
                        ErrorCode::RCapabilityDenied,
                        DiagPhase::Exec,
                        budget.span(),
                        "observe budget must evaluate to budget(...) value",
                    )
                })?;

                validate_ctx_literal(ctx_lit).map_err(|message| {
                    Diagnostic::new(
                        ErrorCode::RCtxInvalid,
                        DiagPhase::Runtime,
                        ctx.span(),
                        message,
                    )
                    .with_root_reason(ReasonCode::CtxInvalid)
                })?;

                self.trace.push(TraceEvent::ObserveStart {
                    key: key_lit.to_string(),
                    tier: tier_lit.to_string(),
                    ctx: ctx_lit.to_string(),
                    budget: budget_units,
                });

                let origin_id = self.allocate_origin_id();
                let r = match self.registry.check_observe(key_lit, ctx_lit) {
                    Ok(kref) => observe_stub_result(&kref.raw, ctx_lit),
                    Err(e) => Result4::<Value>::insufficient(e.to_reason_code()),
                }
                .with_origin_id(origin_id);

                self.observations.insert(
                    origin_id,
                    ObservationMeta {
                        key: key_lit.to_string(),
                    },
                );
                self.trace.push(TraceEvent::ObserveEnd {
                    key: key_lit.to_string(),
                    kind: r.kind,
                    reason: r.reason,
                    origin_id,
                });
                self.env.insert(bind.clone(), Value::Result4(Box::new(r)));
                Ok(Flow::Continue)
            }
            Stmt::Commit { value, span } => {
                self.exec_commit(value, *span)?;
                Ok(Flow::Continue)
            }
            Stmt::Condition { value, span } => {
                self.exec_condition(value, *span)?;
                Ok(Flow::Continue)
            }
            Stmt::Entangle {
                left,
                right,
                constraint,
                span,
            } => {
                self.exec_entangle(left, right, constraint, *span)?;
                Ok(Flow::Continue)
            }
            Stmt::Match(m) => self.exec_match(m, in_function),
        }
    }

    fn exec_entangle(
        &mut self,
        left: &str,
        right: &str,
        constraint: &Expr,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if !self.env.contains_key(left) || !self.env.contains_key(right) {
            return Err(Diagnostic::new(
                ErrorCode::XEntangleBindingUnknown,
                DiagPhase::Exec,
                span,
                "entangle(...) binding is not found in current runtime env",
            )
            .with_root_reason(ReasonCode::KeyUnknown));
        }

        let constraint_value = self.eval_expr(constraint)?;
        let condition = match constraint_value {
            Value::Bool(v) => v,
            _ => {
                return Err(Diagnostic::new(
                    ErrorCode::XEntangleConstraintType,
                    DiagPhase::Exec,
                    span,
                    "entangle(...) constraint must evaluate to bool at runtime",
                )
                .with_root_reason(ReasonCode::AdapterFailed));
            }
        };

        if !condition {
            return Err(Diagnostic::new(
                ErrorCode::XEntangleConstraintFalse,
                DiagPhase::Exec,
                span,
                "entangle(...) constraint evaluated to false",
            ));
        }

        if self.constraints.edge_count >= ENTANGLE_MAX_EDGES_PER_SESSION {
            return Err(Diagnostic::new(
                ErrorCode::XEntangleEdgeCap,
                DiagPhase::Exec,
                span,
                "entangle(...) exceeded session edge cap",
            )
            .with_root_reason(ReasonCode::PolicyDenied));
        }

        let left_degree = self
            .constraints
            .degree_by_binding
            .get(left)
            .copied()
            .unwrap_or(0);
        let right_degree = self
            .constraints
            .degree_by_binding
            .get(right)
            .copied()
            .unwrap_or(0);

        if left_degree >= ENTANGLE_MAX_DEGREE_PER_BINDING
            || right_degree >= ENTANGLE_MAX_DEGREE_PER_BINDING
        {
            return Err(Diagnostic::new(
                ErrorCode::XEntangleDegreeCap,
                DiagPhase::Exec,
                span,
                "entangle(...) exceeded degree cap per binding",
            )
            .with_root_reason(ReasonCode::PolicyDenied));
        }

        let pseudo_elapsed_ns = (self.constraints.edge_count as u64)
            .saturating_add(1)
            .saturating_mul(200);
        if pseudo_elapsed_ns > ENTANGLE_PROPAGATION_BUDGET_NS {
            return Err(Diagnostic::new(
                ErrorCode::XEntangleDeferred,
                DiagPhase::Exec,
                span,
                "entangle(...) deferred due to propagation budget cap",
            )
            .with_root_reason(ReasonCode::BudgetExceeded));
        }

        self.constraints.edge_count = self.constraints.edge_count.saturating_add(1);
        self.constraints
            .degree_by_binding
            .entry(left.to_string())
            .and_modify(|v| *v = v.saturating_add(1))
            .or_insert(1);
        self.constraints
            .degree_by_binding
            .entry(right.to_string())
            .and_modify(|v| *v = v.saturating_add(1))
            .or_insert(1);

        Ok(())
    }

    fn exec_for_range(
        &mut self,
        var: &str,
        start: &Expr,
        end: &Expr,
        body: &[Stmt],
        span: Span,
        in_function: bool,
    ) -> Result<Flow, Diagnostic> {
        let start_v = self.eval_expr(start)?;
        let end_v = self.eval_expr(end)?;
        let Value::Int(start_i) = start_v else {
            return Err(Diagnostic::new(
                ErrorCode::XCommitForbidden,
                DiagPhase::Exec,
                span,
                "for-range start must evaluate to int",
            ));
        };
        let Value::Int(end_i) = end_v else {
            return Err(Diagnostic::new(
                ErrorCode::XCommitForbidden,
                DiagPhase::Exec,
                span,
                "for-range end must evaluate to int",
            ));
        };
        if end_i < start_i {
            return Err(Diagnostic::new(
                ErrorCode::XCommitForbidden,
                DiagPhase::Exec,
                span,
                "for-range end must be >= start",
            ));
        }
        if (end_i - start_i) > 10_000 {
            return Err(Diagnostic::new(
                ErrorCode::XBudgetExceeded,
                DiagPhase::Exec,
                span,
                "for-range exceeds bounded cap (max 10000 iterations)",
            ));
        }
        let snapshot = self.env.clone();
        for i in start_i..end_i {
            self.env.insert(var.to_string(), Value::Int(i));
            if let Flow::Return(v) = self.exec_block(body, in_function)? {
                self.env = snapshot;
                return Ok(Flow::Return(v));
            }
        }
        self.env = snapshot;
        Ok(Flow::Continue)
    }

    fn exec_match(&mut self, m: &MatchStmt, in_function: bool) -> Result<Flow, Diagnostic> {
        let v = self.eval_expr(&m.value)?;
        let Value::Result4(r) = v else {
            return Err(Diagnostic::new(
                ErrorCode::XCommitForbidden,
                DiagPhase::Exec,
                m.span,
                "match expects Result4 value at runtime",
            ));
        };

        self.trace
            .push(TraceEvent::MatchArmSelected { arm: r.kind });
        match r.kind {
            ResultKind::Ok => self.exec_block(&m.ok_arm, in_function),
            ResultKind::Degraded => self.exec_block(&m.degraded_arm, in_function),
            ResultKind::Insufficient => self.exec_block(&m.insufficient_arm, in_function),
            ResultKind::Deferred => self.exec_block(&m.deferred_arm, in_function),
        }
    }

    fn exec_condition(&mut self, expr: &Expr, span: Span) -> Result<(), Diagnostic> {
        let outcome = self.eval_condition_outcome(expr)?;
        match outcome {
            ConditionOutcome::Pass => {
                self.trace.push(TraceEvent::ConditionCheck { value: true });
                Ok(())
            }
            ConditionOutcome::Fail => {
                self.trace.push(TraceEvent::ConditionCheck { value: false });
                Err(Diagnostic::new(
                    ErrorCode::XConditionFalse,
                    DiagPhase::Exec,
                    span,
                    "condition(...) evaluated to false",
                ))
            }
            ConditionOutcome::Deferred(reason) => {
                self.trace.push(TraceEvent::ConditionCheck { value: false });
                Err(Diagnostic::new(
                    ErrorCode::XConditionDeferred,
                    DiagPhase::Exec,
                    span,
                    "condition(...) deferred due to bounded evaluator budget",
                )
                .with_root_reason(reason))
            }
            ConditionOutcome::Insufficient(reason) => {
                self.trace.push(TraceEvent::ConditionCheck { value: false });
                Err(Diagnostic::new(
                    ErrorCode::XConditionInsufficient,
                    DiagPhase::Exec,
                    span,
                    "condition(...) insufficient: evaluator cannot prove bool outcome",
                )
                .with_root_reason(reason))
            }
        }
    }

    fn eval_condition_outcome(&mut self, expr: &Expr) -> Result<ConditionOutcome, Diagnostic> {
        let stats = condition_budget_stats(expr);

        if stats.constraints > CONDITION_MAX_CONSTRAINTS {
            return Ok(ConditionOutcome::Insufficient(ReasonCode::NotImplemented));
        }
        if stats.steps > CONDITION_MAX_STEPS {
            return Ok(ConditionOutcome::Deferred(ReasonCode::BudgetExceeded));
        }
        let pseudo_elapsed_ns = (stats.steps as u64).saturating_mul(400);
        if pseudo_elapsed_ns > CONDITION_TIME_BUDGET_NS {
            return Ok(ConditionOutcome::Deferred(ReasonCode::BudgetExceeded));
        }

        let value = self.eval_expr(expr)?;
        match value {
            Value::Bool(true) => Ok(ConditionOutcome::Pass),
            Value::Bool(false) => Ok(ConditionOutcome::Fail),
            _ => Ok(ConditionOutcome::Insufficient(ReasonCode::AdapterFailed)),
        }
    }

    fn exec_commit(&mut self, expr: &Expr, span: Span) -> Result<(), Diagnostic> {
        let v = self.eval_expr(expr)?;
        let Value::Result4(r) = v else {
            self.trace.push(TraceEvent::CommitAttempt {
                origin_id: None,
                kind: None,
            });
            self.trace.push(TraceEvent::CommitResult {
                allowed: false,
                reason: Some(ReasonCode::PolicyDenied),
            });
            return Err(Diagnostic::new(
                ErrorCode::XCommitForbidden,
                DiagPhase::Exec,
                span,
                "commit(...) expects Result4 value",
            ));
        };

        self.trace.push(TraceEvent::CommitAttempt {
            origin_id: r.origin_id,
            kind: Some(r.kind),
        });

        let Some(origin_id) = r.origin_id else {
            self.trace.push(TraceEvent::CommitResult {
                allowed: false,
                reason: Some(ReasonCode::PolicyDenied),
            });
            return Err(Diagnostic::new(
                ErrorCode::XCommitForbidden,
                DiagPhase::Exec,
                span,
                "commit(...) requires observe origin_id",
            ));
        };
        let Some(meta) = self.observations.get(&origin_id) else {
            self.trace.push(TraceEvent::CommitResult {
                allowed: false,
                reason: Some(ReasonCode::PolicyDenied),
            });
            return Err(Diagnostic::new(
                ErrorCode::XCommitForbidden,
                DiagPhase::Exec,
                span,
                "commit(...) origin_id is not recognized by runtime",
            ));
        };

        match r.kind {
            ResultKind::Ok | ResultKind::Degraded => {
                if !self.registry.commit_allowed_for_key(&meta.key) {
                    self.trace.push(TraceEvent::CommitResult {
                        allowed: false,
                        reason: Some(ReasonCode::PolicyDenied),
                    });
                    return Err(Diagnostic::new(
                        ErrorCode::XCommitForbidden,
                        DiagPhase::Exec,
                        span,
                        "commit policy denied for key",
                    ));
                }
                self.commits.push(CommitEvent {
                    origin_id,
                    key: meta.key.clone(),
                    kind: r.kind,
                });
                self.trace.push(TraceEvent::CommitResult {
                    allowed: true,
                    reason: None,
                });
                Ok(())
            }
            ResultKind::Insufficient | ResultKind::Deferred => {
                self.trace.push(TraceEvent::CommitResult {
                    allowed: false,
                    reason: Some(ReasonCode::PolicyDenied),
                });
                Err(Diagnostic::new(
                    ErrorCode::XCommitForbidden,
                    DiagPhase::Exec,
                    span,
                    "commit(...) is allowed only for OK/DEGRADED result",
                ))
            }
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, Diagnostic> {
        self.tick(expr.span())?;
        match expr {
            Expr::Int { value, .. } => Ok(Value::Int(*value)),
            Expr::Bool { value, .. } => Ok(Value::Bool(*value)),
            Expr::String { value, .. } => Ok(Value::String(value.clone())),
            Expr::Ident { name, span } => self.env.get(name).cloned().ok_or_else(|| {
                Diagnostic::new(
                    ErrorCode::XCommitForbidden,
                    DiagPhase::Exec,
                    *span,
                    format!("unknown runtime identifier `{name}`"),
                )
            }),
            Expr::Call { callee, args, span } => self.eval_call(callee, args, *span),
            Expr::List { items, .. } => {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    out.push(self.eval_expr(item)?);
                }
                Ok(Value::List(out))
            }
            Expr::Map { entries, .. } => {
                let mut out = BTreeMap::new();
                for (k, vexpr) in entries {
                    out.insert(k.clone(), self.eval_expr(vexpr)?);
                }
                Ok(Value::Map(out))
            }
            Expr::Try { value, span } => {
                let base = self.eval_expr(value)?;
                let Value::Result4(result) = base else {
                    return Err(Diagnostic::new(
                        ErrorCode::XCommitForbidden,
                        DiagPhase::Exec,
                        *span,
                        "`?` expects Result4 runtime value",
                    ));
                };
                match result.kind {
                    ResultKind::Ok | ResultKind::Degraded => {
                        Ok(result.payload.unwrap_or(Value::Unit))
                    }
                    ResultKind::Insufficient | ResultKind::Deferred => Err(Diagnostic::new(
                        ErrorCode::XCommitForbidden,
                        DiagPhase::Exec,
                        *span,
                        "`?` cannot unwrap INSUFFICIENT/DEFERRED result",
                    )),
                }
            }
            Expr::FieldAccess { base, field, span } => {
                let b = self.eval_expr(base)?;
                match b {
                    Value::Payload(map) => Ok(map
                        .get(field)
                        .map(|v| Value::String(v.clone()))
                        .unwrap_or(Value::Unknown)),
                    Value::Map(map) => Ok(map.get(field).cloned().unwrap_or(Value::Unknown)),
                    Value::List(items) if field == "len" => Ok(Value::Int(items.len() as i64)),
                    Value::Result4(r) => match r.payload {
                        Some(Value::Payload(map)) => Ok(map
                            .get(field)
                            .map(|v| Value::String(v.clone()))
                            .unwrap_or(Value::Unknown)),
                        _ => Ok(Value::Unknown),
                    },
                    _ => Err(Diagnostic::new(
                        ErrorCode::XCommitForbidden,
                        DiagPhase::Exec,
                        *span,
                        "field access is not supported on this runtime value",
                    )),
                }
            }
        }
    }

    fn eval_call(&mut self, callee: &str, args: &[Expr], span: Span) -> Result<Value, Diagnostic> {
        match callee {
            "budget" => {
                if args.len() != 1 {
                    return Err(Diagnostic::new(
                        ErrorCode::XCommitForbidden,
                        DiagPhase::Exec,
                        span,
                        "budget(...) expects 1 argument",
                    ));
                }
                let arg = self.eval_expr(&args[0])?;
                match arg {
                    Value::Int(v) if v >= 0 => Ok(Value::Budget(v as u32)),
                    _ => Err(Diagnostic::new(
                        ErrorCode::XCommitForbidden,
                        DiagPhase::Exec,
                        span,
                        "budget(...) expects non-negative int",
                    )),
                }
            }
            "ctx" => {
                if args.len() != 1 {
                    return Err(Diagnostic::new(
                        ErrorCode::XCommitForbidden,
                        DiagPhase::Exec,
                        span,
                        "ctx(...) expects 1 argument",
                    ));
                }
                let arg = self.eval_expr(&args[0])?;
                match arg {
                    Value::String(v) => Ok(Value::Ctx(v)),
                    _ => Err(Diagnostic::new(
                        ErrorCode::XCommitForbidden,
                        DiagPhase::Exec,
                        span,
                        "ctx(...) expects string",
                    )),
                }
            }
            "payload" => {
                if args.is_empty() {
                    return Ok(Value::Payload(BTreeMap::new()));
                }
                Err(Diagnostic::new(
                    ErrorCode::XCommitForbidden,
                    DiagPhase::Exec,
                    span,
                    "payload() in V1-D accepts 0 args only",
                ))
            }
            "list" => {
                if !args.is_empty() {
                    return Err(Diagnostic::new(
                        ErrorCode::XCommitForbidden,
                        DiagPhase::Exec,
                        span,
                        "list() expects 0 arguments",
                    ));
                }
                Ok(Value::List(Vec::new()))
            }
            "map" => {
                if !args.is_empty() {
                    return Err(Diagnostic::new(
                        ErrorCode::XCommitForbidden,
                        DiagPhase::Exec,
                        span,
                        "map() expects 0 arguments",
                    ));
                }
                Ok(Value::Map(BTreeMap::new()))
            }
            _ => self.eval_user_function_call(callee, args, span),
        }
    }

    fn eval_user_function_call(
        &mut self,
        callee: &str,
        args: &[Expr],
        span: Span,
    ) -> Result<Value, Diagnostic> {
        let Some(def) = self.functions.get(callee).cloned() else {
            return Err(Diagnostic::new(
                ErrorCode::XCommitForbidden,
                DiagPhase::Exec,
                span,
                format!("unknown runtime call `{callee}`"),
            ));
        };
        if def.params.len() != args.len() {
            return Err(Diagnostic::new(
                ErrorCode::XCommitForbidden,
                DiagPhase::Exec,
                span,
                format!(
                    "function `{callee}` expects {} args, got {}",
                    def.params.len(),
                    args.len()
                ),
            ));
        }

        let mut arg_values = Vec::with_capacity(args.len());
        for arg in args {
            arg_values.push(self.eval_expr(arg)?);
        }

        let snapshot = self.env.clone();
        for (idx, param) in def.params.iter().enumerate() {
            self.env.insert(param.clone(), arg_values[idx].clone());
        }

        let out = match self.exec_block(&def.body, true)? {
            Flow::Continue => Value::Unit,
            Flow::Return(v) => v,
        };

        self.env = snapshot;
        Ok(out)
    }

    fn tick(&mut self, span: Span) -> Result<(), Diagnostic> {
        self.meter.tick().map_err(|_| {
            Diagnostic::new(
                ErrorCode::XBudgetExceeded,
                DiagPhase::Exec,
                span,
                "step cap exceeded",
            )
        })
    }

    fn allocate_origin_id(&mut self) -> u64 {
        let id = self.next_origin_id;
        self.next_origin_id = self.next_origin_id.saturating_add(1);
        id
    }
}

fn stmt_span(stmt: &Stmt) -> Span {
    match stmt {
        Stmt::ModuleDecl { span, .. }
        | Stmt::ImportDecl { span, .. }
        | Stmt::ExportDecl { span, .. }
        | Stmt::StructDecl { span, .. }
        | Stmt::EnumDecl { span, .. }
        | Stmt::FnDef { span, .. }
        | Stmt::Let { span, .. }
        | Stmt::Return { span, .. }
        | Stmt::ForRange { span, .. }
        | Stmt::Observe { span, .. }
        | Stmt::Commit { span, .. }
        | Stmt::Condition { span, .. }
        | Stmt::Entangle { span, .. } => *span,
        Stmt::Match(m) => m.span,
    }
}

fn as_string_runtime(value: &Value) -> Option<&str> {
    match value {
        Value::String(v) => Some(v),
        _ => None,
    }
}

fn as_ctx_runtime(value: &Value) -> Option<&str> {
    match value {
        Value::Ctx(v) => Some(v),
        _ => None,
    }
}

fn as_budget_runtime(value: &Value) -> Option<u32> {
    match value {
        Value::Budget(v) => Some(*v),
        _ => None,
    }
}

fn condition_budget_stats(expr: &Expr) -> ConditionBudgetStats {
    fn walk(expr: &Expr) -> ConditionBudgetStats {
        match expr {
            Expr::Int { .. } | Expr::Bool { .. } | Expr::String { .. } | Expr::Ident { .. } => {
                ConditionBudgetStats {
                    steps: 1,
                    constraints: 1,
                }
            }
            Expr::Call { args, .. } => {
                let mut stats = ConditionBudgetStats {
                    steps: 1,
                    constraints: 1,
                };
                for arg in args {
                    let child = walk(arg);
                    stats.steps = stats.steps.saturating_add(child.steps);
                    stats.constraints = stats.constraints.saturating_add(child.constraints);
                }
                stats
            }
            Expr::List { items, .. } => {
                let mut stats = ConditionBudgetStats {
                    steps: 1,
                    constraints: 1,
                };
                for item in items {
                    let child = walk(item);
                    stats.steps = stats.steps.saturating_add(child.steps);
                    stats.constraints = stats.constraints.saturating_add(child.constraints);
                }
                stats
            }
            Expr::Map { entries, .. } => {
                let mut stats = ConditionBudgetStats {
                    steps: 1,
                    constraints: 1,
                };
                for (_, value) in entries {
                    let child = walk(value);
                    stats.steps = stats.steps.saturating_add(child.steps);
                    stats.constraints = stats.constraints.saturating_add(child.constraints);
                }
                stats
            }
            Expr::Try { value, .. } => {
                let child = walk(value);
                ConditionBudgetStats {
                    steps: child.steps.saturating_add(1),
                    constraints: child.constraints.saturating_add(1),
                }
            }
            Expr::FieldAccess { base, .. } => {
                let child = walk(base);
                ConditionBudgetStats {
                    steps: child.steps.saturating_add(1),
                    constraints: child.constraints.saturating_add(1),
                }
            }
        }
    }

    walk(expr)
}

fn validate_ctx_literal(raw: &str) -> Result<(), String> {
    if raw.is_empty() {
        return Err("ctx(...) must not be empty".to_string());
    }

    let mut seen = HashSet::new();
    for pair in raw.split(';') {
        let Some((key, value)) = pair.split_once('=') else {
            return Err("ctx(...) must use `key=value` pairs separated by `;`".to_string());
        };
        if key.is_empty() || value.is_empty() {
            return Err("ctx(...) key/value must not be empty".to_string());
        }
        if !key
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '-'))
        {
            return Err("ctx(...) key contains invalid character".to_string());
        }
        if !seen.insert(key) {
            return Err("ctx(...) has duplicate key".to_string());
        }
    }
    Ok(())
}

fn observe_stub_result(key: &str, ctx_literal: &str) -> Result4<Value> {
    if key.starts_with("world.ok") {
        return Result4::ok(stub_payload(key));
    }
    if key.starts_with("world.degraded") {
        return Result4::degraded(stub_payload(key), ReasonCode::AdapterFailed);
    }

    let ctx = parse_ctx_pairs(ctx_literal);

    match key {
        "std.args.get" => {
            let mut map = BTreeMap::new();
            let name = ctx
                .get("value")
                .cloned()
                .unwrap_or_else(|| "value".to_string());
            map.insert("key".to_string(), name.clone());
            map.insert("value".to_string(), format!("arg:{name}"));
            Result4::ok(Value::Payload(map))
        }
        "std.fs.read" => {
            let mut map = BTreeMap::new();
            let path = ctx
                .get("path")
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());
            map.insert("path".to_string(), path.clone());
            map.insert("content".to_string(), format!("sample:{path}"));
            Result4::ok(Value::Payload(map))
        }
        "std.fs.write" => {
            let mut map = BTreeMap::new();
            map.insert(
                "path".to_string(),
                ctx.get("path").cloned().unwrap_or_default(),
            );
            map.insert(
                "bytes".to_string(),
                ctx.get("content")
                    .map(|v| v.len().to_string())
                    .unwrap_or_else(|| "0".to_string()),
            );
            map.insert("status".to_string(), "queued".to_string());
            Result4::ok(Value::Payload(map))
        }
        "std.fs.list" => {
            let mut map = BTreeMap::new();
            map.insert(
                "path".to_string(),
                ctx.get("path").cloned().unwrap_or_default(),
            );
            map.insert("items".to_string(), "a.txt,b.txt".to_string());
            Result4::ok(Value::Payload(map))
        }
        "std.fs.stat" => {
            let mut map = BTreeMap::new();
            map.insert(
                "path".to_string(),
                ctx.get("path").cloned().unwrap_or_default(),
            );
            map.insert("exists".to_string(), "true".to_string());
            map.insert("kind".to_string(), "file".to_string());
            Result4::ok(Value::Payload(map))
        }
        "std.http.get" => {
            let mut map = BTreeMap::new();
            map.insert(
                "url".to_string(),
                ctx.get("url").cloned().unwrap_or_default(),
            );
            map.insert("status".to_string(), "200".to_string());
            map.insert("body".to_string(), "{\"ok\":true}".to_string());
            Result4::ok(Value::Payload(map))
        }
        "std.http.post" => {
            let mut map = BTreeMap::new();
            map.insert(
                "url".to_string(),
                ctx.get("url").cloned().unwrap_or_default(),
            );
            map.insert("status".to_string(), "202".to_string());
            map.insert("ack".to_string(), "accepted".to_string());
            Result4::degraded(Value::Payload(map), ReasonCode::AdapterFailed)
        }
        "std.json.parse" => {
            let mut map = BTreeMap::new();
            let raw = ctx.get("raw").cloned().unwrap_or_default();
            map.insert("raw".to_string(), raw);
            map.insert("parsed".to_string(), "true".to_string());
            Result4::ok(Value::Payload(map))
        }
        "std.json.emit" => {
            let mut map = BTreeMap::new();
            let value = ctx.get("value").cloned().unwrap_or_default();
            map.insert("value".to_string(), value.clone());
            map.insert("json".to_string(), format!("{{\"value\":\"{value}\"}}"));
            Result4::ok(Value::Payload(map))
        }
        "std.time.now" => {
            let mut map = BTreeMap::new();
            map.insert("unix_ms".to_string(), "1700000000000".to_string());
            Result4::ok(Value::Payload(map))
        }
        "std.time.sleep" => Result4::deferred(ReasonCode::BudgetExceeded),
        "std.log.info" => {
            let mut map = BTreeMap::new();
            map.insert(
                "message".to_string(),
                ctx.get("message").cloned().unwrap_or_default(),
            );
            map.insert("status".to_string(), "accepted".to_string());
            Result4::ok(Value::Payload(map))
        }
        _ => Result4::deferred(ReasonCode::NotImplemented),
    }
}

fn stub_payload(key: &str) -> Value {
    let mut map = BTreeMap::new();
    map.insert("key".to_string(), key.to_string());
    map.insert("status".to_string(), "sample".to_string());
    Value::Payload(map)
}

fn parse_ctx_pairs(ctx_literal: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for pair in ctx_literal.split(';') {
        let p = pair.trim();
        if p.is_empty() {
            continue;
        }
        if let Some((k, v)) = p.split_once('=') {
            let key = k.trim();
            let val = v.trim();
            if !key.is_empty() {
                out.insert(key.to_string(), val.to_string());
            }
        }
    }
    out
}
