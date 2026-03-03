use std::collections::{BTreeMap, HashMap, HashSet};
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::ocp_ocl::ast::{Expr, MatchStmt, Program, Stmt};
use crate::ocp_ocl::audit::{TraceEvent, TraceLog};
use crate::ocp_ocl::budget::{BudgetMeter, CommitPolicyMode, ExecConfig, GuardMode};
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
    pending_sqlite_write: Option<PendingSqliteWrite>,
}

#[derive(Debug, Clone)]
struct PendingSqliteWrite {
    dsn: String,
    sql: String,
    pending_write_id: String,
}

const CONDITION_TIME_BUDGET_NS: u64 = 20_000;
const CONDITION_MAX_STEPS: usize = 64;
const CONDITION_MAX_CONSTRAINTS: usize = 256;
const LOOP_CAP: u32 = 10_000;
const ENTANGLE_MAX_EDGES_PER_SESSION: usize = 128;
const ENTANGLE_MAX_DEGREE_PER_BINDING: usize = 16;
const ENTANGLE_PROPAGATION_BUDGET_NS: u64 = 20_000;
const VALUE_KEYS_CAP: usize = 2_000;

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
    commit_policy: CommitPolicyMode,
    guard_mode: GuardMode,
    next_origin_id: u64,
    registry: CapabilityRegistry,
    observations: HashMap<u64, ObservationMeta>,
    applied_sqlite_writes: HashSet<String>,
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
            commit_policy: config.commit_policy,
            guard_mode: config.guard_mode,
            next_origin_id: 1,
            registry: CapabilityRegistry::default(),
            observations: HashMap::new(),
            applied_sqlite_writes: HashSet::new(),
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
            commit_policy: config.commit_policy,
            guard_mode: config.guard_mode,
            next_origin_id: 1,
            registry,
            observations: HashMap::new(),
            applied_sqlite_writes: HashSet::new(),
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
                Flow::Return(v) => {
                    self.env.insert("__program_return".to_string(), v);
                    break;
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
            Stmt::Return { value, span: _ } => {
                let v = self.eval_expr(value)?;
                Ok(Flow::Return(v))
            }
            Stmt::TryLet {
                name,
                value,
                else_expr,
                else_returns,
                span,
            } => self.exec_try_let(name, value, else_expr, *else_returns, *span, in_function),
            Stmt::Guard { value, span } => self.exec_guard(value, *span, in_function),
            Stmt::Repeat { count, body, span } => self.exec_repeat(count, body, *span, in_function),
            Stmt::ForEachCap {
                var,
                iter,
                cap,
                body,
                span,
            } => self.exec_for_each_cap(var, iter, cap, body, *span, in_function),
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

                let pending_sqlite_write = extract_pending_sqlite_write(key_lit, &r);
                self.observations.insert(
                    origin_id,
                    ObservationMeta {
                        key: key_lit.to_string(),
                        pending_sqlite_write,
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

    fn exec_try_let(
        &mut self,
        name: &str,
        value: &Expr,
        else_expr: &Expr,
        else_returns: bool,
        span: Span,
        _in_function: bool,
    ) -> Result<Flow, Diagnostic> {
        let base = self.eval_expr(value)?;
        let Value::Result4(result) = base else {
            return Err(Diagnostic::new(
                ErrorCode::XCommitForbidden,
                DiagPhase::Exec,
                span,
                "try/else expects Result4 runtime value",
            ));
        };

        self.trace.push(TraceEvent::MatchArmSelected { arm: result.kind });
        match result.kind {
            ResultKind::Ok | ResultKind::Degraded => {
                // Keep sugar trace/step parity with canonical desugar:
                // match { ... => { let x = rs?; } }
                self.tick(span)?;
                let unwrapped = self.eval_expr(&Expr::Try {
                    value: Box::new(value.clone()),
                    span: value.span(),
                })?;
                self.env.insert(name.to_string(), unwrapped);
                Ok(Flow::Continue)
            }
            ResultKind::Insufficient | ResultKind::Deferred => {
                // Keep sugar trace/step parity with canonical desugar:
                // match { ... => { let x = <else>; } / { return <else>; } }
                self.tick(span)?;
                let prev_r = self.env.insert(
                    "r".to_string(),
                    Value::Result4(Box::new((*result).clone())),
                );
                let else_value = self.eval_expr(else_expr);
                match prev_r {
                    Some(v) => {
                        self.env.insert("r".to_string(), v);
                    }
                    None => {
                        self.env.remove("r");
                    }
                }
                let else_value = else_value?;
                if else_returns {
                    Ok(Flow::Return(else_value))
                } else {
                    self.env.insert(name.to_string(), else_value);
                    Ok(Flow::Continue)
                }
            }
        }
    }

    fn exec_guard(
        &mut self,
        value: &Expr,
        span: Span,
        _in_function: bool,
    ) -> Result<Flow, Diagnostic> {
        let base = self.eval_expr(value)?;
        let Value::Result4(result) = base else {
            return Err(Diagnostic::new(
                ErrorCode::XCommitForbidden,
                DiagPhase::Exec,
                span,
                "guard expects Result4 runtime value",
            ));
        };

        self.trace.push(TraceEvent::MatchArmSelected { arm: result.kind });
        match result.kind {
            ResultKind::Ok | ResultKind::Degraded => Ok(Flow::Continue),
            ResultKind::Insufficient | ResultKind::Deferred => match self.guard_mode {
                GuardMode::Return => {
                    // Keep sugar trace/step parity with canonical desugar:
                    // match { INSUFFICIENT/DEFERRED => { return rs; } }
                    self.tick(span)?;
                    let ret = self.eval_expr(value)?;
                    Ok(Flow::Return(ret))
                }
                GuardMode::Error => Err(
                    Diagnostic::new(
                        ErrorCode::XGuardFailed,
                        DiagPhase::Exec,
                        span,
                        "guard failed on non-success Result4",
                    )
                    .with_root_reason(result.reason.unwrap_or(ReasonCode::PolicyDenied)),
                ),
            },
        }
    }

    fn exec_repeat(
        &mut self,
        count: &Expr,
        body: &[Stmt],
        span: Span,
        in_function: bool,
    ) -> Result<Flow, Diagnostic> {
        let count_v = self.eval_expr(count)?;
        let Value::Int(count_i) = count_v else {
            return Err(Diagnostic::new(
                ErrorCode::XCommitForbidden,
                DiagPhase::Exec,
                span,
                "repeat count must evaluate to int",
            ));
        };
        if count_i < 0 {
            return Err(Diagnostic::new(
                ErrorCode::XCommitForbidden,
                DiagPhase::Exec,
                span,
                "repeat count must be non-negative",
            ));
        }
        let count_u = count_i as u32;
        if count_u > LOOP_CAP {
            return Err(loop_cap_exceeded(
                span,
                "repeat count exceeds configured loop cap",
            ));
        }

        let snapshot = self.env.clone();
        for i in 0..count_u {
            self.trace.push(TraceEvent::LoopIter {
                loop_kind: "repeat".to_string(),
                iter_index: i,
            });
            if let Flow::Return(v) = self.exec_block(body, in_function)? {
                self.env = snapshot;
                return Ok(Flow::Return(v));
            }
        }
        self.env = snapshot;
        Ok(Flow::Continue)
    }

    fn exec_for_each_cap(
        &mut self,
        var: &str,
        iter: &Expr,
        cap: &Expr,
        body: &[Stmt],
        span: Span,
        in_function: bool,
    ) -> Result<Flow, Diagnostic> {
        let iter_v = self.eval_expr(iter)?;
        let Value::List(items) = iter_v else {
            return Err(Diagnostic::new(
                ErrorCode::XCommitForbidden,
                DiagPhase::Exec,
                span,
                "for ... cap iterable must evaluate to list",
            ));
        };

        let cap_v = self.eval_expr(cap)?;
        let Value::Int(cap_i) = cap_v else {
            return Err(Diagnostic::new(
                ErrorCode::XCommitForbidden,
                DiagPhase::Exec,
                span,
                "for ... cap must evaluate to int",
            ));
        };
        if cap_i < 0 {
            return Err(Diagnostic::new(
                ErrorCode::XCommitForbidden,
                DiagPhase::Exec,
                span,
                "for ... cap must be non-negative",
            ));
        }

        let cap_u = cap_i as u32;
        if cap_u > LOOP_CAP {
            return Err(loop_cap_exceeded(
                span,
                "for ... cap exceeds configured loop cap",
            ));
        }

        let limit = usize::min(items.len(), cap_u as usize);
        let snapshot = self.env.clone();
        for (idx, item) in items.into_iter().take(limit).enumerate() {
            self.trace.push(TraceEvent::LoopIter {
                loop_kind: "for_cap".to_string(),
                iter_index: idx as u32,
            });
            self.env.insert(var.to_string(), item);
            if let Flow::Return(v) = self.exec_block(body, in_function)? {
                self.env = snapshot;
                return Ok(Flow::Return(v));
            }
        }
        self.env = snapshot;
        Ok(Flow::Continue)
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
        if (end_i - start_i) > i64::from(LOOP_CAP) {
            return Err(loop_cap_exceeded(
                span,
                "for-range exceeds configured loop cap",
            ));
        }
        let snapshot = self.env.clone();
        for (idx, i) in (start_i..end_i).enumerate() {
            self.trace.push(TraceEvent::LoopIter {
                loop_kind: "for_range".to_string(),
                iter_index: idx as u32,
            });
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
        let Some(meta) = self.observations.get(&origin_id).cloned() else {
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
                match self.commit_policy {
                    CommitPolicyMode::Normal => {
                        self.apply_pending_sqlite_write_if_needed(&meta, span)?;
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
                    CommitPolicyMode::ForbidCommit => {
                        self.trace.push(TraceEvent::CommitResult {
                            allowed: false,
                            reason: Some(ReasonCode::PolicyDenied),
                        });
                        Ok(())
                    }
                    CommitPolicyMode::ShadowCommitLog => {
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
                }
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

    fn apply_pending_sqlite_write_if_needed(
        &mut self,
        meta: &ObservationMeta,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let Some(pending) = meta.pending_sqlite_write.as_ref() else {
            return Ok(());
        };
        if !db_local_real_enabled() {
            return Ok(());
        }
        if self
            .applied_sqlite_writes
            .contains(&pending.pending_write_id)
        {
            return Ok(());
        }

        if pending
            .sql
            .trim_start()
            .to_ascii_lowercase()
            .starts_with("select")
        {
            return Err(Diagnostic::new(
                ErrorCode::XCommitForbidden,
                DiagPhase::Exec,
                span,
                "std.db.exec commit does not allow SELECT statement",
            )
            .with_root_reason(ReasonCode::PolicyDenied));
        }

        sqlite_exec_local_real(&pending.dsn, &pending.sql).map_err(|reason| {
            Diagnostic::new(
                ErrorCode::XCommitForbidden,
                DiagPhase::Exec,
                span,
                "std.db.exec local-real apply failed",
            )
            .with_root_reason(reason)
        })?;

        self.applied_sqlite_writes
            .insert(pending.pending_write_id.clone());
        Ok(())
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
                    Value::Result4(r) if field == "kind" => {
                        let text = match r.kind {
                            ResultKind::Ok => "OK",
                            ResultKind::Degraded => "DEGRADED",
                            ResultKind::Insufficient => "INSUFFICIENT",
                            ResultKind::Deferred => "DEFERRED",
                        };
                        Ok(Value::String(text.to_string()))
                    }
                    Value::Result4(r) if field == "reason_code" => Ok(Value::String(
                        r.reason
                            .map(|rc| rc.as_str().to_string())
                            .unwrap_or_default(),
                    )),
                    Value::Result4(_) if field == "audit" => Ok(Value::Map(BTreeMap::new())),
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
            "len" => {
                if args.len() != 1 {
                    return Err(Diagnostic::new(
                        ErrorCode::XCommitForbidden,
                        DiagPhase::Exec,
                        span,
                        "len(...) expects 1 argument",
                    ));
                }
                let arg = self.eval_expr(&args[0])?;
                let len = match arg {
                    Value::List(items) => items.len(),
                    Value::Map(map) => map.len(),
                    Value::Payload(map) => map.len(),
                    Value::String(s) => s.chars().count(),
                    _ => {
                        return Err(Diagnostic::new(
                            ErrorCode::XCommitForbidden,
                            DiagPhase::Exec,
                            span,
                            "len(...) expects list/map/payload/string",
                        ))
                    }
                };
                Ok(Value::Int(len as i64))
            }
            "keys" => {
                if args.len() != 2 {
                    return Err(Diagnostic::new(
                        ErrorCode::XCommitForbidden,
                        DiagPhase::Exec,
                        span,
                        "keys(...) expects 2 arguments",
                    ));
                }
                let map_value = self.eval_expr(&args[0])?;
                let cap_value = self.eval_expr(&args[1])?;
                let cap = as_non_negative_int_runtime(&cap_value).ok_or_else(|| {
                    Diagnostic::new(
                        ErrorCode::XCommitForbidden,
                        DiagPhase::Exec,
                        span,
                        "keys(...) cap must be non-negative int",
                    )
                })?;
                if cap as usize > VALUE_KEYS_CAP {
                    return Err(value_keys_cap_exceeded(
                        span,
                        "keys(...) cap exceeds value_keys_cap",
                    ));
                }
                let map = into_mapish_runtime(map_value, "keys(...)", span)?;
                let mut out = Vec::new();
                for key in map.keys().take(cap as usize) {
                    out.push(Value::String(key.clone()));
                }
                Ok(Value::List(out))
            }
            "merge" => {
                if args.len() != 3 {
                    return Err(Diagnostic::new(
                        ErrorCode::XCommitForbidden,
                        DiagPhase::Exec,
                        span,
                        "merge(...) expects 3 arguments",
                    ));
                }
                let left = self.eval_expr(&args[0])?;
                let right = self.eval_expr(&args[1])?;
                let cap_value = self.eval_expr(&args[2])?;
                let cap = as_non_negative_int_runtime(&cap_value).ok_or_else(|| {
                    Diagnostic::new(
                        ErrorCode::XCommitForbidden,
                        DiagPhase::Exec,
                        span,
                        "merge(...) cap must be non-negative int",
                    )
                })?;
                if cap as usize > VALUE_KEYS_CAP {
                    return Err(value_keys_cap_exceeded(
                        span,
                        "merge(...) cap exceeds value_keys_cap",
                    ));
                }

                let mut merged = into_mapish_runtime(left, "merge(...)", span)?;
                let right_map = into_mapish_runtime(right, "merge(...)", span)?;
                for (k, v) in right_map {
                    merged.insert(k, v);
                }
                if merged.len() > cap as usize {
                    return Err(value_keys_cap_exceeded(
                        span,
                        "merge(...) result key count exceeds cap",
                    ));
                }
                Ok(Value::Map(merged))
            }
            "std.json.parse" => {
                if args.len() != 1 {
                    return Err(Diagnostic::new(
                        ErrorCode::XCommitForbidden,
                        DiagPhase::Exec,
                        span,
                        "std.json.parse(...) expects 1 argument",
                    ));
                }
                let arg = self.eval_expr(&args[0])?;
                let raw = as_string_runtime(&arg).ok_or_else(|| {
                    Diagnostic::new(
                        ErrorCode::XCommitForbidden,
                        DiagPhase::Exec,
                        span,
                        "std.json.parse(...) argument must evaluate to string",
                    )
                })?;
                let result = match parse_json_value(raw) {
                    Ok(v) => Result4::ok(v),
                    Err(()) => Result4::insufficient(ReasonCode::JsonInvalid),
                };
                Ok(Value::Result4(Box::new(result)))
            }
            "std.json.stringify" => {
                if args.len() != 1 {
                    return Err(Diagnostic::new(
                        ErrorCode::XCommitForbidden,
                        DiagPhase::Exec,
                        span,
                        "std.json.stringify(...) expects 1 argument",
                    ));
                }
                let arg = self.eval_expr(&args[0])?;
                Ok(Value::String(stringify_json_value(&arg)))
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
        | Stmt::TryLet { span, .. }
        | Stmt::Guard { span, .. }
        | Stmt::Repeat { span, .. }
        | Stmt::ForEachCap { span, .. }
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

fn as_non_negative_int_runtime(value: &Value) -> Option<u32> {
    match value {
        Value::Int(v) if *v >= 0 => Some(*v as u32),
        _ => None,
    }
}

fn into_mapish_runtime(
    value: Value,
    fn_name: &str,
    span: Span,
) -> Result<BTreeMap<String, Value>, Diagnostic> {
    match value {
        Value::Map(map) => Ok(map),
        Value::Payload(map) => Ok(map
            .into_iter()
            .map(|(k, v)| (k, Value::String(v)))
            .collect()),
        _ => Err(Diagnostic::new(
            ErrorCode::XCommitForbidden,
            DiagPhase::Exec,
            span,
            format!("{fn_name} expects map/payload arguments"),
        )),
    }
}

fn value_keys_cap_exceeded(span: Span, message: &'static str) -> Diagnostic {
    Diagnostic::new(ErrorCode::XKeysCapExceeded, DiagPhase::Exec, span, message)
        .with_root_reason(ReasonCode::PolicyDenied)
        .with_alias("X-LIMIT-EXCEEDED")
        .with_limit_kind("value_keys_cap")
        .with_hint("reduce cap/result key count to fit value_keys_cap")
}

fn loop_cap_exceeded(span: Span, message: &'static str) -> Diagnostic {
    Diagnostic::new(ErrorCode::XLoopCapExceeded, DiagPhase::Exec, span, message)
        .with_root_reason(ReasonCode::PolicyDenied)
        .with_alias("X-LIMIT-EXCEEDED")
        .with_limit_kind("loop_cap")
        .with_hint("reduce loop iteration cap to fit configured loop_cap")
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

fn extract_pending_sqlite_write(key: &str, result: &Result4<Value>) -> Option<PendingSqliteWrite> {
    if key != "std.db.exec" || !matches!(result.kind, ResultKind::Ok | ResultKind::Degraded) {
        return None;
    }
    let Value::Payload(map) = result.payload.as_ref()? else {
        return None;
    };
    let dsn = map.get("dsn")?.clone();
    let sql = map.get("sql")?.clone();
    let pending_write_id = map.get("pending_write_id")?.clone();
    Some(PendingSqliteWrite {
        dsn,
        sql,
        pending_write_id,
    })
}

fn flag_enabled(var_name: &str) -> bool {
    match env::var(var_name) {
        Ok(raw) => matches!(
            raw.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes"
        ),
        Err(_) => false,
    }
}

fn tls_local_real_enabled() -> bool {
    flag_enabled("OCL_W7_TLS_LOCAL_REAL")
}

fn db_local_real_enabled() -> bool {
    flag_enabled("OCL_W7_DB_LOCAL_REAL")
}

fn tls_local_timeout() -> Duration {
    let millis = env::var("OCL_W7_TLS_TIMEOUT_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(1_500);
    Duration::from_millis(millis)
}

fn tls_local_probe(host: &str, port: &str) -> Result<(), ReasonCode> {
    let addr = format!("{host}:{port}");
    let mut addrs = std::net::ToSocketAddrs::to_socket_addrs(addr.as_str())
        .map_err(|_| ReasonCode::AdapterFailed)?;
    let socket = addrs.next().ok_or(ReasonCode::AdapterFailed)?;
    TcpStream::connect_timeout(&socket, tls_local_timeout())
        .map(|_| ())
        .map_err(|_| ReasonCode::AdapterFailed)
}

fn sqlite_path_from_dsn(dsn: &str) -> Result<String, ReasonCode> {
    let trimmed = dsn.trim();
    if trimmed.is_empty() {
        return Err(ReasonCode::CtxInvalid);
    }
    if trimmed == "memory:" || trimmed == ":memory:" {
        return Ok(":memory:".to_string());
    }
    let Some(raw_path) = trimmed.strip_prefix("file:") else {
        return Err(ReasonCode::CtxInvalid);
    };
    if raw_path.trim().is_empty() {
        return Err(ReasonCode::CtxInvalid);
    }
    Ok(raw_path.to_string())
}

fn sqlite_query_int_local_real(dsn: &str, sql: &str) -> Result<i64, ReasonCode> {
    let db_path = sqlite_path_from_dsn(dsn)?;
    if db_path != ":memory:" {
        let path = std::path::Path::new(&db_path);
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|_| ReasonCode::AdapterFailed)?;
            }
        }
    }
    let script = concat!(
        "import sqlite3,sys\n",
        "db,sql = sys.argv[1],sys.argv[2]\n",
        "conn = sqlite3.connect(db)\n",
        "try:\n",
        "  cur = conn.execute(sql)\n",
        "  row = cur.fetchone()\n",
        "  if row is None or row[0] is None:\n",
        "    print('0')\n",
        "  else:\n",
        "    print(str(row[0]))\n",
        "finally:\n",
        "  conn.close()\n",
    );
    let output = Command::new("python")
        .args(["-c", script, db_path.as_str(), sql])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|_| ReasonCode::AdapterFailed)?;
    if !output.status.success() {
        return Err(ReasonCode::AdapterFailed);
    }
    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    raw.parse::<i64>().map_err(|_| ReasonCode::AdapterFailed)
}

fn sqlite_exec_local_real(dsn: &str, sql: &str) -> Result<(), ReasonCode> {
    let db_path = sqlite_path_from_dsn(dsn)?;
    if db_path != ":memory:" {
        let path = std::path::Path::new(&db_path);
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|_| ReasonCode::AdapterFailed)?;
            }
        }
    }
    let script = concat!(
        "import sqlite3,sys\n",
        "db,sql = sys.argv[1],sys.argv[2]\n",
        "conn = sqlite3.connect(db)\n",
        "try:\n",
        "  conn.executescript(sql)\n",
        "  conn.commit()\n",
        "finally:\n",
        "  conn.close()\n",
    );
    let status = Command::new("python")
        .args(["-c", script, db_path.as_str(), sql])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|_| ReasonCode::AdapterFailed)?;
    if !status.success() {
        return Err(ReasonCode::AdapterFailed);
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
    if key.starts_with("custom.") {
        return observe_custom_plugin_result(key, ctx_literal);
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
        "std.net.listen" => {
            let mut map = BTreeMap::new();
            map.insert(
                "addr".to_string(),
                ctx.get("addr").cloned().unwrap_or_default(),
            );
            map.insert("state".to_string(), "listening".to_string());
            map.insert("event_id".to_string(), "1".to_string());
            Result4::ok(Value::Payload(map))
        }
        "std.net.reply" => {
            let mut map = BTreeMap::new();
            map.insert(
                "conn".to_string(),
                ctx.get("conn").cloned().unwrap_or_default(),
            );
            map.insert(
                "status".to_string(),
                ctx.get("status")
                    .cloned()
                    .unwrap_or_else(|| "200".to_string()),
            );
            map.insert(
                "bytes".to_string(),
                ctx.get("body")
                    .map(|v| v.len().to_string())
                    .unwrap_or_else(|| "0".to_string()),
            );
            map.insert("queued".to_string(), "true".to_string());
            Result4::ok(Value::Payload(map))
        }
        "std.net.close" => {
            let mut map = BTreeMap::new();
            map.insert(
                "conn".to_string(),
                ctx.get("conn").cloned().unwrap_or_default(),
            );
            map.insert("closed".to_string(), "true".to_string());
            Result4::degraded(Value::Payload(map), ReasonCode::AdapterFailed)
        }
        "std.tls.connect" => {
            let mut map = BTreeMap::new();
            let host = ctx.get("host").cloned().unwrap_or_default();
            let port = ctx
                .get("port")
                .cloned()
                .unwrap_or_else(|| "443".to_string());
            let conn = format!("{host}:{port}");
            if tls_local_real_enabled() {
                if tls_local_probe(&host, &port).is_err() {
                    return Result4::deferred(ReasonCode::AdapterFailed);
                }
                map.insert("mode".to_string(), "local_real".to_string());
                map.insert("state".to_string(), "connected_local_real".to_string());
            } else {
                map.insert("mode".to_string(), "stub".to_string());
                map.insert("state".to_string(), "connected".to_string());
            }
            map.insert("host".to_string(), host);
            map.insert("port".to_string(), port);
            map.insert("conn".to_string(), conn);
            Result4::ok(Value::Payload(map))
        }
        "std.tls.handshake" => {
            let mut map = BTreeMap::new();
            let conn = ctx.get("conn").cloned().unwrap_or_default();
            let (host, port) = match conn.split_once(':') {
                Some((h, p)) => (h.to_string(), p.to_string()),
                None => (conn.clone(), "443".to_string()),
            };
            if tls_local_real_enabled() && tls_local_probe(&host, &port).is_err() {
                return Result4::deferred(ReasonCode::AdapterFailed);
            }
            map.insert("conn".to_string(), conn);
            map.insert("protocol".to_string(), "TLS1.3".to_string());
            map.insert("cipher".to_string(), "platform-default".to_string());
            map.insert(
                "mode".to_string(),
                if tls_local_real_enabled() {
                    "local_real".to_string()
                } else {
                    "stub".to_string()
                },
            );
            map.insert(
                "transcript_hash256".to_string(),
                stable_hash256_hex(&format!("{}|{}|{}", host, port, tls_local_real_enabled())),
            );
            Result4::ok(Value::Payload(map))
        }
        "std.db.query_int" => {
            let mut map = BTreeMap::new();
            let sql = ctx.get("sql").cloned().unwrap_or_default();
            let dsn = ctx.get("dsn").cloned().unwrap_or_default();
            map.insert("dsn".to_string(), dsn);
            map.insert("sql".to_string(), sql.clone());
            let value = if db_local_real_enabled() {
                match sqlite_query_int_local_real(&map["dsn"], &sql) {
                    Ok(v) => v.to_string(),
                    Err(reason) => return Result4::deferred(reason),
                }
            } else if sql.to_ascii_lowercase().contains("count") {
                "1".to_string()
            } else {
                "0".to_string()
            };
            map.insert("value".to_string(), value);
            map.insert(
                "mode".to_string(),
                if db_local_real_enabled() {
                    "local_real".to_string()
                } else {
                    "stub".to_string()
                },
            );
            Result4::ok(Value::Payload(map))
        }
        "std.db.exec" => {
            let mut map = BTreeMap::new();
            let dsn = ctx.get("dsn").cloned().unwrap_or_default();
            let sql = ctx.get("sql").cloned().unwrap_or_default();
            let pending_write_id = stable_hash64_hex(&format!("{dsn}|{sql}"));
            map.insert("dsn".to_string(), dsn);
            map.insert("sql".to_string(), sql);
            map.insert("pending_write_id".to_string(), pending_write_id);
            map.insert("applied".to_string(), "false".to_string());
            map.insert(
                "mode".to_string(),
                if db_local_real_enabled() {
                    "local_real".to_string()
                } else {
                    "stub".to_string()
                },
            );
            Result4::ok(Value::Payload(map))
        }
        "std.view.render_text" => {
            let mut map = BTreeMap::new();
            let truth = ctx.get("truth").cloned().unwrap_or_default();
            let view_id = resolve_active_view_id(&ctx);
            let renderer = "text";
            let rendered = format!("[{view_id}] {truth}");
            map.insert("truth".to_string(), truth);
            map.insert("view_id".to_string(), view_id.clone());
            map.insert("renderer".to_string(), renderer.to_string());
            map.insert("rendered".to_string(), rendered);
            map.insert(
                "render_hash256".to_string(),
                stable_hash256_hex(&format!("{renderer}|{view_id}|{}", map["truth"])),
            );
            Result4::ok(Value::Payload(map))
        }
        "std.view.render_tree" => {
            let mut map = BTreeMap::new();
            let truth = ctx.get("truth").cloned().unwrap_or_default();
            let view_id = resolve_active_view_id(&ctx);
            let renderer = "tree";
            let tree_hash = stable_hash64_hex(&format!("tree|{view_id}|{truth}"));
            let rendered = format!("root(view={view_id}, hash={tree_hash})");
            map.insert("truth".to_string(), truth);
            map.insert("view_id".to_string(), view_id.clone());
            map.insert("renderer".to_string(), renderer.to_string());
            map.insert("rendered".to_string(), rendered);
            map.insert(
                "render_hash256".to_string(),
                stable_hash256_hex(&format!("{renderer}|{view_id}|{}", map["truth"])),
            );
            Result4::ok(Value::Payload(map))
        }
        "std.ui.frame_info" => {
            let mut map = BTreeMap::new();
            let tick = ctx
                .get("ctx_tick")
                .cloned()
                .unwrap_or_else(|| "0".to_string());
            map.insert("ctx_tick".to_string(), tick.clone());
            map.insert("frame".to_string(), tick);
            Result4::ok(Value::Payload(map))
        }
        "std.game.tick_info" => {
            let mut map = BTreeMap::new();
            let tick = ctx
                .get("ctx_tick")
                .cloned()
                .unwrap_or_else(|| "0".to_string());
            map.insert("ctx_tick".to_string(), tick.clone());
            map.insert("tick".to_string(), tick);
            Result4::ok(Value::Payload(map))
        }
        _ => Result4::deferred(ReasonCode::NotImplemented),
    }
}

#[derive(Debug, Clone)]
struct PluginLockEntryRuntime {
    id: String,
    key_prefix: String,
    command: String,
}

fn observe_custom_plugin_result(key: &str, ctx_literal: &str) -> Result4<Value> {
    let entries = match load_plugin_lock_entries_from_env() {
        Ok(v) => v,
        Err(reason) => return Result4::deferred(reason),
    };
    if entries.is_empty() {
        return Result4::deferred(ReasonCode::PluginUnavailable);
    }

    let selected = entries
        .iter()
        .filter(|entry| key.starts_with(&entry.key_prefix))
        .max_by_key(|entry| entry.key_prefix.len());
    let Some(entry) = selected else {
        return Result4::insufficient(ReasonCode::KeyUnknown);
    };

    let response = match invoke_plugin_jsonl(entry, key, ctx_literal) {
        Ok(v) => v,
        Err(reason) => return Result4::deferred(reason),
    };

    let kind = json_field_string(&response, "kind")
        .unwrap_or_else(|| "deferred".to_string())
        .to_ascii_lowercase();
    let reason = json_field_string(&response, "reason").and_then(|v| reason_code_from_str(&v));
    let payload_text = json_field_string(&response, "payload").unwrap_or_default();

    let mut payload = BTreeMap::new();
    payload.insert("plugin_id".to_string(), entry.id.clone());
    payload.insert("key".to_string(), key.to_string());
    payload.insert("payload".to_string(), payload_text);

    match kind.as_str() {
        "ok" => Result4::ok(Value::Payload(payload)),
        "degraded" => Result4::degraded(
            Value::Payload(payload),
            reason.unwrap_or(ReasonCode::AdapterFailed),
        ),
        "insufficient" => Result4::insufficient(reason.unwrap_or(ReasonCode::AdapterFailed)),
        "deferred" => Result4::deferred(reason.unwrap_or(ReasonCode::AdapterFailed)),
        _ => Result4::deferred(ReasonCode::PluginProtocolError),
    }
}

fn load_plugin_lock_entries_from_env() -> Result<Vec<PluginLockEntryRuntime>, ReasonCode> {
    let lock_path = env::var("OCL_PLUGIN_LOCK_PATH").map_err(|_| ReasonCode::PluginUnavailable)?;
    let raw = std::fs::read_to_string(&lock_path).map_err(|_| ReasonCode::PluginUnavailable)?;
    let mut out = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed == "version=1" {
            continue;
        }
        let Some(row) = trimmed.strip_prefix("plugin=") else {
            continue;
        };
        let parts: Vec<&str> = row.split('\t').collect();
        if parts.len() != 6 {
            return Err(ReasonCode::PluginProtocolError);
        }
        out.push(PluginLockEntryRuntime {
            id: parts[0].to_string(),
            key_prefix: parts[1].to_string(),
            command: parts[3].to_string(),
        });
    }
    Ok(out)
}

fn invoke_plugin_jsonl(
    entry: &PluginLockEntryRuntime,
    key: &str,
    ctx_literal: &str,
) -> Result<String, ReasonCode> {
    let req_line = format!(
        "{{\"id\":\"1\",\"key\":\"{}\",\"ctx\":\"{}\"}}\n",
        json_escape_inline(key),
        json_escape_inline(ctx_literal)
    );
    let started = Instant::now();

    let mut command = spawn_shell_command(&entry.command);
    if let Ok(root) = env::var("OCL_PLUGIN_ROOT") {
        command.current_dir(root);
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| ReasonCode::PluginUnavailable)?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(req_line.as_bytes())
            .map_err(|_| ReasonCode::PluginProtocolError)?;
        stdin.flush().map_err(|_| ReasonCode::PluginProtocolError)?;
    } else {
        return Err(ReasonCode::PluginProtocolError);
    }

    let mut line = String::new();
    if let Some(stdout) = child.stdout.take() {
        let mut reader = BufReader::new(stdout);
        let read = reader
            .read_line(&mut line)
            .map_err(|_| ReasonCode::PluginProtocolError)?;
        if read == 0 {
            return Err(ReasonCode::PluginProtocolError);
        }
    } else {
        return Err(ReasonCode::PluginProtocolError);
    }

    let _ = child.wait();

    if started.elapsed() > Duration::from_millis(500) {
        return Err(ReasonCode::PluginTimeout);
    }
    Ok(line.trim().to_string())
}

fn spawn_shell_command(command_line: &str) -> Command {
    #[cfg(target_os = "windows")]
    {
        let mut command = Command::new("cmd");
        command.args(["/C", command_line]);
        command
    }
    #[cfg(not(target_os = "windows"))]
    {
        let mut command = Command::new("sh");
        command.args(["-lc", command_line]);
        command
    }
}

fn json_field_string(line: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\":");
    let idx = line.find(&needle)?;
    let mut rest = line[idx + needle.len()..].trim_start();
    if let Some(v) = rest.strip_prefix('"') {
        rest = v;
        let end = rest.find('"')?;
        return Some(rest[..end].to_string());
    }
    None
}

fn parse_json_value(raw: &str) -> Result<Value, ()> {
    let mut p = JsonParser {
        src: raw.as_bytes(),
        idx: 0,
    };
    p.skip_ws();
    let v = p.parse_value()?;
    p.skip_ws();
    if p.idx != p.src.len() {
        return Err(());
    }
    Ok(v)
}

struct JsonParser<'a> {
    src: &'a [u8],
    idx: usize,
}

impl<'a> JsonParser<'a> {
    fn peek(&self) -> Option<u8> {
        self.src.get(self.idx).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let b = self.peek()?;
        self.idx += 1;
        Some(b)
    }

    fn skip_ws(&mut self) {
        while let Some(b) = self.peek() {
            if matches!(b, b' ' | b'\n' | b'\r' | b'\t') {
                self.idx += 1;
            } else {
                break;
            }
        }
    }

    fn parse_value(&mut self) -> Result<Value, ()> {
        self.skip_ws();
        match self.peek() {
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b'"') => self.parse_string().map(Value::String),
            Some(b't') => {
                self.expect_keyword(b"true")?;
                Ok(Value::Bool(true))
            }
            Some(b'f') => {
                self.expect_keyword(b"false")?;
                Ok(Value::Bool(false))
            }
            Some(b'n') => {
                self.expect_keyword(b"null")?;
                Ok(Value::Unit)
            }
            Some(b'-') | Some(b'0'..=b'9') => self.parse_int().map(Value::Int),
            _ => Err(()),
        }
    }

    fn expect_keyword(&mut self, kw: &[u8]) -> Result<(), ()> {
        if self.src.get(self.idx..self.idx + kw.len()) == Some(kw) {
            self.idx += kw.len();
            Ok(())
        } else {
            Err(())
        }
    }

    fn parse_string(&mut self) -> Result<String, ()> {
        if self.bump() != Some(b'"') {
            return Err(());
        }
        let mut out = String::new();
        loop {
            let b = self.bump().ok_or(())?;
            match b {
                b'"' => return Ok(out),
                b'\\' => {
                    let esc = self.bump().ok_or(())?;
                    match esc {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        _ => return Err(()),
                    }
                }
                b if b < 0x20 => return Err(()),
                b => out.push(b as char),
            }
        }
    }

    fn parse_int(&mut self) -> Result<i64, ()> {
        let start = self.idx;
        if self.peek() == Some(b'-') {
            self.idx += 1;
        }
        let mut has_digit = false;
        while let Some(b'0'..=b'9') = self.peek() {
            has_digit = true;
            self.idx += 1;
        }
        if !has_digit {
            return Err(());
        }
        if matches!(self.peek(), Some(b'.' | b'e' | b'E')) {
            return Err(());
        }
        let s = std::str::from_utf8(&self.src[start..self.idx]).map_err(|_| ())?;
        s.parse::<i64>().map_err(|_| ())
    }

    fn parse_array(&mut self) -> Result<Value, ()> {
        if self.bump() != Some(b'[') {
            return Err(());
        }
        self.skip_ws();
        let mut items = Vec::new();
        if self.peek() == Some(b']') {
            self.idx += 1;
            return Ok(Value::List(items));
        }
        loop {
            items.push(self.parse_value()?);
            self.skip_ws();
            match self.bump() {
                Some(b',') => {
                    self.skip_ws();
                }
                Some(b']') => break,
                _ => return Err(()),
            }
        }
        Ok(Value::List(items))
    }

    fn parse_object(&mut self) -> Result<Value, ()> {
        if self.bump() != Some(b'{') {
            return Err(());
        }
        self.skip_ws();
        let mut map = BTreeMap::new();
        if self.peek() == Some(b'}') {
            self.idx += 1;
            return Ok(Value::Map(map));
        }
        loop {
            let key = self.parse_string()?;
            self.skip_ws();
            if self.bump() != Some(b':') {
                return Err(());
            }
            self.skip_ws();
            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_ws();
            match self.bump() {
                Some(b',') => {
                    self.skip_ws();
                }
                Some(b'}') => break,
                _ => return Err(()),
            }
        }
        Ok(Value::Map(map))
    }
}

fn stringify_json_value(value: &Value) -> String {
    match value {
        Value::Int(v) => v.to_string(),
        Value::Bool(v) => {
            if *v {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        Value::String(v) => format!("\"{}\"", escape_json_string(v)),
        Value::List(items) => {
            let mut out = String::from("[");
            for (idx, item) in items.iter().enumerate() {
                if idx > 0 {
                    out.push(',');
                }
                out.push_str(&stringify_json_value(item));
            }
            out.push(']');
            out
        }
        Value::Map(map) => {
            let mut out = String::from("{");
            for (idx, (k, v)) in map.iter().enumerate() {
                if idx > 0 {
                    out.push(',');
                }
                out.push('"');
                out.push_str(&escape_json_string(k));
                out.push_str("\":");
                out.push_str(&stringify_json_value(v));
            }
            out.push('}');
            out
        }
        Value::Payload(map) => {
            let mut out = String::from("{");
            for (idx, (k, v)) in map.iter().enumerate() {
                if idx > 0 {
                    out.push(',');
                }
                out.push('"');
                out.push_str(&escape_json_string(k));
                out.push_str("\":\"");
                out.push_str(&escape_json_string(v));
                out.push('"');
            }
            out.push('}');
            out
        }
        Value::Result4(r) => {
            let kind = match r.kind {
                ResultKind::Ok => "OK",
                ResultKind::Degraded => "DEGRADED",
                ResultKind::Insufficient => "INSUFFICIENT",
                ResultKind::Deferred => "DEFERRED",
            };
            let reason = r.reason.map(|v| v.as_str()).unwrap_or("");
            let payload = r
                .payload
                .as_ref()
                .map(stringify_json_value)
                .unwrap_or_else(|| "null".to_string());
            format!(
                "{{\"kind\":\"{}\",\"reason_code\":\"{}\",\"payload\":{}}}",
                kind, reason, payload
            )
        }
        Value::Budget(v) => v.to_string(),
        Value::Ctx(v) => format!("\"{}\"", escape_json_string(v)),
        Value::Unit | Value::Unknown => "null".to_string(),
    }
}

fn escape_json_string(input: &str) -> String {
    let mut out = String::new();
    for ch in input.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push(' '),
            c => out.push(c),
        }
    }
    out
}

fn reason_code_from_str(raw: &str) -> Option<ReasonCode> {
    match raw {
        "RC-BUDGET-EXCEEDED" => Some(ReasonCode::BudgetExceeded),
        "RC-CAPABILITY-DENIED" => Some(ReasonCode::CapabilityDenied),
        "RC-KEY-UNKNOWN" => Some(ReasonCode::KeyUnknown),
        "RC-CTX-INVALID" => Some(ReasonCode::CtxInvalid),
        "RC-POLICY-DENIED" => Some(ReasonCode::PolicyDenied),
        "RC-ADAPTER-FAILED" => Some(ReasonCode::AdapterFailed),
        "RC-JSON-INVALID" => Some(ReasonCode::JsonInvalid),
        "RC-NOT-IMPLEMENTED" => Some(ReasonCode::NotImplemented),
        "RC-PLUGIN-PROTOCOL-ERROR" => Some(ReasonCode::PluginProtocolError),
        "RC-PLUGIN-UNAVAILABLE" => Some(ReasonCode::PluginUnavailable),
        "RC-PLUGIN-TIMEOUT" => Some(ReasonCode::PluginTimeout),
        _ => None,
    }
}

fn json_escape_inline(input: &str) -> String {
    let mut out = String::new();
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push(' '),
            c => out.push(c),
        }
    }
    out
}

fn stable_hash256_hex(input: &str) -> String {
    let a = stable_hash64_hex(&format!("0|{input}"));
    let b = stable_hash64_hex(&format!("1|{input}"));
    let c = stable_hash64_hex(&format!("2|{input}"));
    let d = stable_hash64_hex(&format!("3|{input}"));
    format!("{a}{b}{c}{d}")
}

fn stable_hash64_hex(input: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in input.as_bytes() {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
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

fn resolve_active_view_id(ctx: &HashMap<String, String>) -> String {
    if let Some(view_id) = ctx.get("view_id") {
        if !view_id.trim().is_empty() {
            return view_id.to_string();
        }
    }
    match env::var("OCL_VIEW_ID") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => "default".to_string(),
    }
}
