use std::collections::{BTreeMap, HashMap, HashSet};
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::path::{Component, Path, PathBuf};
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
    pending_kv_write: Option<PendingKvWrite>,
    pending_fs_write: Option<PendingFsWrite>,
    pending_ui_write: Option<PendingUiWrite>,
    pending_game_write: Option<PendingGameWrite>,
}

#[derive(Debug, Clone)]
struct PendingSqliteWrite {
    dsn: String,
    sql: String,
    pending_write_id: String,
}

#[derive(Debug, Clone)]
struct PendingKvWrite {
    op: KvCommitOp,
    pending_write_id: String,
}

#[derive(Debug, Clone)]
enum KvCommitOp {
    Put {
        key: String,
        value: Value,
        overwrite: bool,
    },
    Del {
        key: String,
    },
    Clear {
        prefix: Option<String>,
    },
}

#[derive(Debug, Clone)]
struct PendingFsWrite {
    op: FsCommitOp,
    pending_write_id: String,
}

#[derive(Debug, Clone)]
enum FsCommitOp {
    WriteText {
        path: String,
        text: String,
        overwrite: bool,
    },
    Mkdir {
        path: String,
        recursive: bool,
    },
    Remove {
        path: String,
        recursive: bool,
    },
    Rename {
        from: String,
        to: String,
        overwrite: bool,
    },
}

#[derive(Debug, Clone)]
struct PendingUiWrite {
    op: UiCommitOp,
    pending_write_id: String,
}

#[derive(Debug, Clone)]
enum UiCommitOp {
    Draw { cmd_count: usize },
    Present,
}

#[derive(Debug, Clone)]
struct PendingGameWrite {
    op: GameCommitOp,
    pending_write_id: String,
}

#[derive(Debug, Clone)]
enum GameCommitOp {
    StateDelta {
        idempotency_key: String,
        delta: Value,
        delta_bytes: usize,
    },
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
    applied_kv_writes: HashSet<String>,
    applied_fs_writes: HashSet<String>,
    applied_ui_writes: HashSet<String>,
    applied_game_writes: HashSet<String>,
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
            applied_kv_writes: HashSet::new(),
            applied_fs_writes: HashSet::new(),
            applied_ui_writes: HashSet::new(),
            applied_game_writes: HashSet::new(),
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
            applied_kv_writes: HashSet::new(),
            applied_fs_writes: HashSet::new(),
            applied_ui_writes: HashSet::new(),
            applied_game_writes: HashSet::new(),
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
                let pending_kv_write = extract_pending_kv_write(key_lit, &r);
                let pending_fs_write = extract_pending_fs_write(key_lit, &r);
                let pending_ui_write = extract_pending_ui_write(key_lit, &r);
                let pending_game_write = extract_pending_game_write(key_lit, &r);
                self.observations.insert(
                    origin_id,
                    ObservationMeta {
                        key: key_lit.to_string(),
                        pending_sqlite_write,
                        pending_kv_write,
                        pending_fs_write,
                        pending_ui_write,
                        pending_game_write,
                    },
                );
                self.trace.push(TraceEvent::ObserveEnd {
                    key: key_lit.to_string(),
                    kind: r.kind,
                    reason: r.reason,
                    origin_id,
                });
                self.emit_ui_observe_trace(key_lit, &r);
                self.emit_game_observe_trace(key_lit, &r);
                self.emit_shadow_observe_trace(key_lit, &r);
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

        self.trace
            .push(TraceEvent::MatchArmSelected { arm: result.kind });
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
                let prev_r = self
                    .env
                    .insert("r".to_string(), Value::Result4(Box::new((*result).clone())));
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

        self.trace
            .push(TraceEvent::MatchArmSelected { arm: result.kind });
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
                GuardMode::Error => Err(Diagnostic::new(
                    ErrorCode::XGuardFailed,
                    DiagPhase::Exec,
                    span,
                    "guard failed on non-success Result4",
                )
                .with_root_reason(result.reason.unwrap_or(ReasonCode::PolicyDenied))),
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
                        self.apply_pending_kv_write_if_needed(&meta, span)?;
                        self.apply_pending_fs_write_if_needed(&meta, span)?;
                        self.apply_pending_ui_write_if_needed(&meta, r.kind, r.reason);
                        self.apply_pending_game_write_if_needed(&meta, r.kind, r.reason);
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
                        self.apply_pending_ui_write_if_needed(&meta, r.kind, r.reason);
                        self.apply_pending_game_write_if_needed(&meta, r.kind, r.reason);
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

    fn apply_pending_kv_write_if_needed(
        &mut self,
        meta: &ObservationMeta,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let Some(pending) = meta.pending_kv_write.as_ref() else {
            return Ok(());
        };
        if self.applied_kv_writes.contains(&pending.pending_write_id) {
            return Ok(());
        }

        let mut store = load_kv_store().map_err(|reason| {
            Diagnostic::new(
                ErrorCode::XCommitForbidden,
                DiagPhase::Exec,
                span,
                "std.kv commit cannot load state store",
            )
            .with_root_reason(reason)
        })?;

        match &pending.op {
            KvCommitOp::Put {
                key,
                value,
                overwrite,
            } => {
                if !overwrite && store.contains_key(key) {
                    return Err(Diagnostic::new(
                        ErrorCode::XCommitForbidden,
                        DiagPhase::Exec,
                        span,
                        "std.kv.put commit denied: key exists and overwrite=false",
                    )
                    .with_root_reason(ReasonCode::PolicyDenied));
                }
                if !store.contains_key(key) && store.len() >= kv_max_keys() {
                    return Err(Diagnostic::new(
                        ErrorCode::XCommitForbidden,
                        DiagPhase::Exec,
                        span,
                        "std.kv.put commit deferred: max_keys exceeded",
                    )
                    .with_root_reason(ReasonCode::KvCapExceeded));
                }
                if stringify_json_value(value).len() > kv_max_value_bytes() {
                    return Err(Diagnostic::new(
                        ErrorCode::XCommitForbidden,
                        DiagPhase::Exec,
                        span,
                        "std.kv.put commit deferred: value exceeds max_value_bytes",
                    )
                    .with_root_reason(ReasonCode::KvCapExceeded));
                }
                store.insert(key.clone(), value.clone());
            }
            KvCommitOp::Del { key } => {
                store.remove(key);
            }
            KvCommitOp::Clear { prefix } => {
                if let Some(prefix) = prefix {
                    store.retain(|k, _| !k.starts_with(prefix));
                } else {
                    store.clear();
                }
            }
        }

        save_kv_store(&store).map_err(|reason| {
            Diagnostic::new(
                ErrorCode::XCommitForbidden,
                DiagPhase::Exec,
                span,
                "std.kv commit cannot persist state store",
            )
            .with_root_reason(reason)
        })?;

        self.applied_kv_writes
            .insert(pending.pending_write_id.clone());
        Ok(())
    }

    fn apply_pending_fs_write_if_needed(
        &mut self,
        meta: &ObservationMeta,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let Some(pending) = meta.pending_fs_write.as_ref() else {
            return Ok(());
        };
        if self.applied_fs_writes.contains(&pending.pending_write_id) {
            return Ok(());
        }

        let apply_result: Result<(), ReasonCode> = (|| match &pending.op {
            FsCommitOp::WriteText {
                path,
                text,
                overwrite,
            } => {
                let (full_path, _) = resolve_fs_path("write", path)?;
                if full_path.exists() && !overwrite {
                    Err(ReasonCode::PolicyDenied)
                } else if text.len() > fs_max_write_bytes() {
                    Err(ReasonCode::LimitExceeded)
                } else {
                    fs_write_text_atomic(&full_path, text)
                }
            }
            FsCommitOp::Mkdir { path, recursive } => {
                let (full_path, _) = resolve_fs_path("write", path)?;
                if *recursive {
                    fs::create_dir_all(&full_path).map_err(|_| ReasonCode::FsIoError)
                } else {
                    fs::create_dir(&full_path).map_err(|_| ReasonCode::FsIoError)
                }
            }
            FsCommitOp::Remove { path, recursive } => {
                let (full_path, _) = resolve_fs_path("remove", path)?;
                if !full_path.exists() {
                    Ok(())
                } else {
                    let meta = fs::metadata(&full_path).map_err(|_| ReasonCode::FsIoError)?;
                    if meta.is_dir() {
                        if *recursive {
                            fs::remove_dir_all(&full_path).map_err(|_| ReasonCode::FsIoError)
                        } else {
                            fs::remove_dir(&full_path).map_err(|_| ReasonCode::FsIoError)
                        }
                    } else {
                        fs::remove_file(&full_path).map_err(|_| ReasonCode::FsIoError)
                    }
                }
            }
            FsCommitOp::Rename {
                from,
                to,
                overwrite,
            } => {
                let (from_full, _) = resolve_fs_path("rename", from)?;
                let (to_full, _) = resolve_fs_path("rename", to)?;
                if !from_full.exists() {
                    Err(ReasonCode::FsNotFound)
                } else if to_full.exists() && !overwrite {
                    Err(ReasonCode::PolicyDenied)
                } else {
                    if to_full.exists() {
                        let to_meta = fs::metadata(&to_full).map_err(|_| ReasonCode::FsIoError)?;
                        if to_meta.is_dir() {
                            fs::remove_dir_all(&to_full).map_err(|_| ReasonCode::FsIoError)?;
                        } else {
                            fs::remove_file(&to_full).map_err(|_| ReasonCode::FsIoError)?;
                        }
                    }
                    if let Some(parent) = to_full.parent() {
                        if !parent.as_os_str().is_empty() {
                            fs::create_dir_all(parent).map_err(|_| ReasonCode::FsIoError)?;
                        }
                    }
                    fs::rename(&from_full, &to_full).map_err(|_| ReasonCode::FsIoError)
                }
            }
        })();

        apply_result.map_err(|reason| {
            Diagnostic::new(
                ErrorCode::XCommitForbidden,
                DiagPhase::Exec,
                span,
                "std.fs commit apply failed",
            )
            .with_root_reason(reason)
        })?;

        self.applied_fs_writes
            .insert(pending.pending_write_id.clone());
        Ok(())
    }

    fn apply_pending_ui_write_if_needed(
        &mut self,
        meta: &ObservationMeta,
        kind: ResultKind,
        reason: Option<ReasonCode>,
    ) {
        let Some(pending) = meta.pending_ui_write.as_ref() else {
            return;
        };
        if self.applied_ui_writes.contains(&pending.pending_write_id) {
            return;
        }

        match &pending.op {
            UiCommitOp::Draw { cmd_count } => {
                self.trace.push(TraceEvent::UiCommit {
                    key: meta.key.clone(),
                    cmd_count: *cmd_count as u32,
                    present: false,
                    kind,
                    reason,
                });
            }
            UiCommitOp::Present => {
                self.trace.push(TraceEvent::UiCommit {
                    key: meta.key.clone(),
                    cmd_count: 0,
                    present: true,
                    kind,
                    reason,
                });
            }
        }

        self.applied_ui_writes
            .insert(pending.pending_write_id.clone());
    }

    fn apply_pending_game_write_if_needed(
        &mut self,
        meta: &ObservationMeta,
        kind: ResultKind,
        reason: Option<ReasonCode>,
    ) {
        let Some(pending) = meta.pending_game_write.as_ref() else {
            return;
        };
        if self.applied_game_writes.contains(&pending.pending_write_id) {
            return;
        }

        match &pending.op {
            GameCommitOp::StateDelta {
                idempotency_key,
                delta,
                delta_bytes,
            } => {
                let delta_hash = stable_hash64_hex(&stringify_json_value(delta));
                self.trace.push(TraceEvent::GameCommit {
                    key: meta.key.clone(),
                    delta_bytes: *delta_bytes as u32,
                    idempotency_hash: stable_hash64_hex(&format!("{idempotency_key}|{delta_hash}")),
                    kind,
                    reason,
                });
            }
        }

        self.applied_game_writes
            .insert(pending.pending_write_id.clone());
    }

    fn emit_ui_observe_trace(&mut self, key: &str, result: &Result4<Value>) {
        if !key.starts_with("std.ui.") {
            return;
        }

        let mut event_count = 0u32;
        let mut truncated = false;
        let detail = result
            .payload
            .as_ref()
            .map(stringify_json_value)
            .unwrap_or_else(|| "null".to_string());

        if key == "std.ui.input" {
            if let Some(Value::Map(map)) = result.payload.as_ref() {
                if let Some(Value::List(events)) = map.get("events") {
                    event_count = events.len() as u32;
                }
                if let Some(Value::Bool(flag)) = map.get("truncated") {
                    truncated = *flag;
                }
            }
        }

        self.trace.push(TraceEvent::UiObserve {
            key: key.to_string(),
            event_count,
            truncated,
            detail,
            kind: result.kind,
            reason: result.reason,
        });
    }

    fn emit_game_observe_trace(&mut self, key: &str, result: &Result4<Value>) {
        if !key.starts_with("std.game.") {
            return;
        }

        let mut stream = String::new();
        let mut value_count = 0u32;
        let mut tick = 0i64;
        if let Some(Value::Map(map)) = result.payload.as_ref() {
            if let Some(Value::String(v)) = map.get("stream") {
                stream = v.clone();
            }
            if let Some(Value::List(values)) = map.get("values") {
                value_count = values.len() as u32;
            }
            if let Some(Value::Int(v)) = map.get("tick") {
                tick = *v;
            }
        } else if let Some(Value::Payload(map)) = result.payload.as_ref() {
            if let Some(raw_tick) = map.get("tick") {
                tick = raw_tick.trim().parse::<i64>().ok().unwrap_or(0);
            }
        }

        self.trace.push(TraceEvent::GameObserve {
            key: key.to_string(),
            stream,
            value_count,
            tick,
            kind: result.kind,
            reason: result.reason,
        });
    }

    fn emit_shadow_observe_trace(&mut self, key: &str, result: &Result4<Value>) {
        if !key.starts_with("std.shadow.") {
            return;
        }

        let Some(Value::Map(map)) = result.payload.as_ref() else {
            return;
        };

        match key {
            "std.shadow.run" => {
                let branch_count = map
                    .get("branches")
                    .and_then(|v| match v {
                        Value::List(items) => Some(items.len() as u32),
                        _ => None,
                    })
                    .unwrap_or(0);
                let truncated = map
                    .get("truncated")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                let detail = map
                    .get("branch_digest")
                    .and_then(Value::as_string)
                    .unwrap_or_default();
                self.trace.push(TraceEvent::ShadowRun {
                    key: key.to_string(),
                    branch_count,
                    truncated,
                    detail,
                    kind: result.kind,
                    reason: result.reason,
                });
            }
            "std.shadow.compare" => {
                let diff_count = map
                    .get("diff_keys")
                    .and_then(|v| match v {
                        Value::List(items) => Some(items.len() as u32),
                        _ => None,
                    })
                    .unwrap_or(0);
                let report_bytes = map
                    .get("report_bytes")
                    .and_then(|v| match v {
                        Value::Int(raw) => Some((*raw).max(0) as u32),
                        _ => None,
                    })
                    .unwrap_or(0);
                let truncated = map
                    .get("truncated")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                self.trace.push(TraceEvent::ShadowCompare {
                    key: key.to_string(),
                    diff_count,
                    report_bytes,
                    truncated,
                    kind: result.kind,
                    reason: result.reason,
                });
            }
            _ => {}
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

fn extract_pending_kv_write(_key: &str, result: &Result4<Value>) -> Option<PendingKvWrite> {
    if !matches!(result.kind, ResultKind::Ok | ResultKind::Degraded) {
        return None;
    }
    let payload = result.payload.as_ref()?;
    let Value::Map(map) = payload else {
        return None;
    };
    let op = map.get("op")?.as_string()?;
    let pending_write_id = map.get("pending_write_id")?.as_string()?;

    let op = match op.as_str() {
        "put" => {
            let key = map.get("key")?.as_string()?;
            let value = map.get("value")?.clone();
            let overwrite = map
                .get("overwrite")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            KvCommitOp::Put {
                key,
                value,
                overwrite,
            }
        }
        "del" => KvCommitOp::Del {
            key: map.get("key")?.as_string()?,
        },
        "clear" => {
            let prefix = match map.get("prefix") {
                Some(Value::String(v)) => Some(v.clone()),
                _ => None,
            };
            KvCommitOp::Clear { prefix }
        }
        _ => return None,
    };

    Some(PendingKvWrite {
        op,
        pending_write_id,
    })
}

fn extract_pending_fs_write(key: &str, result: &Result4<Value>) -> Option<PendingFsWrite> {
    if !matches!(result.kind, ResultKind::Ok | ResultKind::Degraded) {
        return None;
    }
    let Value::Map(map) = result.payload.as_ref()? else {
        return None;
    };
    let pending_write_id = map.get("pending_write_id")?.as_string()?;
    let op = match key {
        "std.fs.write_text" => FsCommitOp::WriteText {
            path: map.get("path")?.as_string()?,
            text: map.get("text")?.as_string()?,
            overwrite: map
                .get("overwrite")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        },
        "std.fs.mkdir" => FsCommitOp::Mkdir {
            path: map.get("path")?.as_string()?,
            recursive: map
                .get("recursive")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        },
        "std.fs.remove" => FsCommitOp::Remove {
            path: map.get("path")?.as_string()?,
            recursive: map
                .get("recursive")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        },
        "std.fs.rename" => FsCommitOp::Rename {
            from: map.get("from")?.as_string()?,
            to: map.get("to")?.as_string()?,
            overwrite: map
                .get("overwrite")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        },
        _ => return None,
    };
    Some(PendingFsWrite {
        op,
        pending_write_id,
    })
}

fn extract_pending_ui_write(key: &str, result: &Result4<Value>) -> Option<PendingUiWrite> {
    if !matches!(key, "std.ui.draw" | "std.ui.present") {
        return None;
    }
    let payload = result.payload.as_ref()?;
    let Value::Map(map) = payload else {
        return None;
    };
    let pending_write_id = map.get("pending_write_id")?.as_string()?;
    let op = match key {
        "std.ui.draw" => {
            let cmd_count = match map.get("cmd_count") {
                Some(Value::Int(v)) => (*v).max(0) as usize,
                Some(Value::String(raw)) => raw.parse::<usize>().ok().unwrap_or(0),
                _ => 0,
            };
            UiCommitOp::Draw { cmd_count }
        }
        "std.ui.present" => UiCommitOp::Present,
        _ => return None,
    };
    Some(PendingUiWrite {
        op,
        pending_write_id,
    })
}

fn extract_pending_game_write(key: &str, result: &Result4<Value>) -> Option<PendingGameWrite> {
    if key != "std.game.state_delta" {
        return None;
    }
    if !matches!(result.kind, ResultKind::Ok | ResultKind::Degraded) {
        return None;
    }
    let Value::Map(map) = result.payload.as_ref()? else {
        return None;
    };
    let pending_write_id = map.get("pending_write_id")?.as_string()?;
    let idempotency_key = map.get("idempotency_key")?.as_string()?;
    let delta = map.get("delta")?.clone();
    let delta_bytes = map
        .get("delta_bytes")
        .and_then(|v| match v {
            Value::Int(raw) => Some((*raw).max(0) as usize),
            Value::String(raw) => raw.trim().parse::<usize>().ok(),
            _ => None,
        })
        .unwrap_or_else(|| stringify_json_value(&delta).len());

    Some(PendingGameWrite {
        op: GameCommitOp::StateDelta {
            idempotency_key,
            delta,
            delta_bytes,
        },
        pending_write_id,
    })
}

fn kv_store_path() -> PathBuf {
    if let Ok(raw) = env::var("OCL_STD_KV_PATH") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    PathBuf::from(".ocl_state/kv.json")
}

fn kv_max_keys() -> usize {
    env::var("OCL_STD_KV_MAX_KEYS")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(5_000)
}

fn kv_max_value_bytes() -> usize {
    env::var("OCL_STD_KV_MAX_VALUE_BYTES")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(65_536)
}

fn load_kv_store() -> Result<BTreeMap<String, Value>, ReasonCode> {
    let path = kv_store_path();
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let raw = fs::read_to_string(&path).map_err(|_| ReasonCode::KvIoError)?;
    if raw.trim().is_empty() {
        return Ok(BTreeMap::new());
    }
    let parsed = parse_json_value(&raw).map_err(|_| ReasonCode::KvIoError)?;
    let Value::Map(map) = parsed else {
        return Err(ReasonCode::KvIoError);
    };
    Ok(map)
}

fn save_kv_store(store: &BTreeMap<String, Value>) -> Result<(), ReasonCode> {
    let path = kv_store_path();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|_| ReasonCode::KvIoError)?;
        }
    }
    let tmp_path = path.with_extension("tmp");
    let content = stringify_json_value(&Value::Map(store.clone()));
    fs::write(&tmp_path, content).map_err(|_| ReasonCode::KvIoError)?;
    fs::rename(&tmp_path, &path).map_err(|_| ReasonCode::KvIoError)?;
    Ok(())
}

fn fs_sandbox_root() -> PathBuf {
    if let Ok(raw) = env::var("OCL_STD_FS_ROOT") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    PathBuf::from(".")
}

fn fs_max_read_bytes() -> usize {
    env::var("OCL_STD_FS_MAX_READ_BYTES")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(1_048_576)
}

fn fs_max_write_bytes() -> usize {
    env::var("OCL_STD_FS_MAX_WRITE_BYTES")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(1_048_576)
}

fn fs_max_list_entries() -> usize {
    env::var("OCL_STD_FS_MAX_LIST_ENTRIES")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(500)
}

fn ui_max_draw_cmds() -> usize {
    env::var("OCL_STD_UI_MAX_DRAW_CMDS")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(5_000)
}

fn ui_max_input_events() -> usize {
    env::var("OCL_STD_UI_MAX_INPUT_EVENTS")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(500)
}

fn game_enabled() -> bool {
    match env::var("OCL_STD_GAME_ENABLED") {
        Ok(raw) => !matches!(
            raw.trim().to_ascii_lowercase().as_str(),
            "0" | "false" | "no"
        ),
        Err(_) => true,
    }
}

fn game_fixed_dt_ms() -> i64 {
    env::var("OCL_STD_GAME_FIXED_DT_MS")
        .ok()
        .and_then(|v| v.trim().parse::<i64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(16)
}

fn game_rng_max_count() -> usize {
    env::var("OCL_STD_GAME_RNG_MAX_COUNT")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(1024)
}

fn game_state_delta_max_bytes() -> usize {
    env::var("OCL_STD_GAME_STATE_DELTA_MAX_BYTES")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(65_536)
}

fn game_rng_streams() -> Vec<String> {
    let raw = env::var("OCL_STD_GAME_RNG_STREAMS").unwrap_or_else(|_| "main,loot".to_string());
    raw.split(',')
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
        .map(|v| v.to_string())
        .collect()
}

fn game_stream_allowed(stream: &str) -> bool {
    game_rng_streams().iter().any(|v| v == stream)
}

fn game_base_seed() -> u64 {
    env::var("OCL_STD_GAME_BASE_SEED")
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .unwrap_or(0)
}

fn shadow_enabled() -> bool {
    match env::var("OCL_STD_SHADOW_ENABLED") {
        Ok(raw) => !matches!(
            raw.trim().to_ascii_lowercase().as_str(),
            "0" | "false" | "no"
        ),
        Err(_) => true,
    }
}

fn shadow_max_branches() -> usize {
    env::var("OCL_STD_SHADOW_MAX_BRANCHES")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(8)
}

fn shadow_branch_step_cap() -> usize {
    env::var("OCL_STD_SHADOW_BRANCH_STEP_CAP")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(5_000)
}

fn shadow_branch_budget_cap() -> usize {
    env::var("OCL_STD_SHADOW_BRANCH_BUDGET_CAP")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(200_000)
}

fn shadow_max_diff_keys() -> usize {
    env::var("OCL_STD_SHADOW_MAX_DIFF_KEYS")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(2_000)
}

fn shadow_max_report_bytes() -> usize {
    env::var("OCL_STD_SHADOW_MAX_REPORT_BYTES")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(262_144)
}

fn shadow_disallowed_effect_keys(ctx: &HashMap<String, String>) -> Vec<String> {
    let mut out = Vec::new();
    let raw = ctx.get("effect_keys").cloned().unwrap_or_default();
    for token in raw.split('|') {
        let key = token.trim();
        if key.is_empty() {
            continue;
        }
        let disallowed = key.starts_with("std.fs.")
            || key.starts_with("std.kv.")
            || key == "std.ui.present"
            || key == "std.game.state_delta";
        if disallowed {
            out.push(key.to_string());
        }
    }
    out
}

fn parse_shadow_variants(ctx: &HashMap<String, String>) -> Result<Vec<Value>, ()> {
    if let Some(raw_json) = ctx.get("variants_json") {
        let parsed = parse_json_value(raw_json)?;
        if let Value::List(items) = parsed {
            return Ok(items);
        }
        return Err(());
    }

    if let Some(raw) = ctx.get("variants") {
        let items = raw
            .split('|')
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
            .map(|v| Value::String(v.to_string()))
            .collect::<Vec<_>>();
        return Ok(items);
    }

    Err(())
}

#[derive(Debug, Clone)]
struct ShadowBranchView {
    id: i64,
    outcome: ResultKind,
    reason: Option<ReasonCode>,
    signature: String,
    cost_steps: i64,
    cost_budget: i64,
    state_summary: Value,
}

fn parse_shadow_compare_branches(
    ctx: &HashMap<String, String>,
) -> Result<Vec<ShadowBranchView>, ()> {
    let raw_json = ctx.get("branches_json").ok_or(())?;
    let parsed = parse_json_value(raw_json)?;
    let Value::List(items) = parsed else {
        return Err(());
    };

    let mut out = Vec::new();
    for item in items {
        let Value::Map(map) = item else {
            continue;
        };
        let id = map
            .get("id")
            .and_then(|v| match v {
                Value::Int(raw) => Some(*raw),
                Value::String(raw) => raw.trim().parse::<i64>().ok(),
                _ => None,
            })
            .unwrap_or(0);
        let outcome = map
            .get("outcome")
            .and_then(|v| match v {
                Value::String(raw) => Some(parse_result_kind_token(raw)),
                _ => None,
            })
            .unwrap_or(ResultKind::Ok);
        let reason = map.get("reason_code").and_then(|v| match v {
            Value::String(raw) => reason_code_from_str(raw),
            _ => None,
        });
        let signature = map
            .get("signature")
            .and_then(Value::as_string)
            .unwrap_or_default();
        let (cost_steps, cost_budget) = match map.get("cost") {
            Some(Value::Map(cost)) => {
                let steps = cost
                    .get("steps")
                    .and_then(|v| match v {
                        Value::Int(raw) => Some(*raw),
                        Value::String(raw) => raw.trim().parse::<i64>().ok(),
                        _ => None,
                    })
                    .unwrap_or(0);
                let budget = cost
                    .get("budget")
                    .and_then(|v| match v {
                        Value::Int(raw) => Some(*raw),
                        Value::String(raw) => raw.trim().parse::<i64>().ok(),
                        _ => None,
                    })
                    .unwrap_or(0);
                (steps.max(0), budget.max(0))
            }
            _ => (0, 0),
        };
        let state_summary = map.get("state_summary").cloned().unwrap_or(Value::Unit);
        out.push(ShadowBranchView {
            id,
            outcome,
            reason,
            signature,
            cost_steps,
            cost_budget,
            state_summary,
        });
    }

    if out.is_empty() {
        return Err(());
    }
    out.sort_by_key(|v| v.id);
    Ok(out)
}

fn parse_result_kind_token(raw: &str) -> ResultKind {
    match raw.trim().to_ascii_uppercase().as_str() {
        "DEGRADED" => ResultKind::Degraded,
        "INSUFFICIENT" => ResultKind::Insufficient,
        "DEFERRED" => ResultKind::Deferred,
        _ => ResultKind::Ok,
    }
}

#[derive(Debug, Clone)]
struct ShadowCompareBuild {
    payload: BTreeMap<String, Value>,
    reason: Option<ReasonCode>,
}

fn build_shadow_compare_report(
    branches: &[ShadowBranchView],
    baseline_id: i64,
    max_diff_keys: usize,
    max_report_bytes: usize,
) -> ShadowCompareBuild {
    let baseline = branches
        .iter()
        .find(|b| b.id == baseline_id)
        .unwrap_or(&branches[0]);
    let mut diff_keys = collect_shadow_diff_keys(&baseline.state_summary, branches);
    let mut reason = None;
    let mut truncated = false;

    if diff_keys.len() > max_diff_keys {
        diff_keys.truncate(max_diff_keys);
        truncated = true;
        reason = Some(ReasonCode::ShadowCapExceeded);
    }

    let mut cost_table = Vec::with_capacity(branches.len());
    for branch in branches {
        let mut row = BTreeMap::new();
        row.insert("id".to_string(), Value::Int(branch.id));
        row.insert(
            "outcome".to_string(),
            Value::String(format!("{:?}", branch.outcome).to_ascii_uppercase()),
        );
        if let Some(code) = branch.reason {
            row.insert(
                "reason_code".to_string(),
                Value::String(code.as_str().to_string()),
            );
        }
        row.insert("steps".to_string(), Value::Int(branch.cost_steps.max(0)));
        row.insert("budget".to_string(), Value::Int(branch.cost_budget.max(0)));
        row.insert(
            "signature".to_string(),
            Value::String(branch.signature.clone()),
        );
        cost_table.push(Value::Map(row));
    }

    let mut reason_counts = BTreeMap::<String, i64>::new();
    for branch in branches {
        if let Some(code) = branch.reason {
            *reason_counts.entry(code.as_str().to_string()).or_insert(0) += 1;
        }
    }
    let mut reason_table = Vec::new();
    for (code, count) in reason_counts {
        let mut row = BTreeMap::new();
        row.insert("reason_code".to_string(), Value::String(code));
        row.insert("count".to_string(), Value::Int(count.max(0)));
        reason_table.push(Value::Map(row));
    }

    let mut diff_keys_for_report = diff_keys;
    let mut report_reason = reason;
    let mut report_truncated = truncated;
    let mut report_bytes = 0usize;
    loop {
        let payload = build_shadow_compare_payload(
            &diff_keys_for_report,
            &cost_table,
            &reason_table,
            report_truncated,
            report_bytes,
        );
        report_bytes = stringify_json_value(&Value::Map(payload.clone())).len();
        if report_bytes <= max_report_bytes {
            let mut stable = payload;
            stable.insert("report_bytes".to_string(), Value::Int(report_bytes as i64));
            return ShadowCompareBuild {
                payload: stable,
                reason: report_reason,
            };
        }

        report_truncated = true;
        report_reason = Some(ReasonCode::ShadowReportTooLarge);
        if !reason_table.is_empty() {
            reason_table.pop();
            continue;
        }
        if diff_keys_for_report.len() > 1 {
            diff_keys_for_report.pop();
            continue;
        }
        if cost_table.len() > 1 {
            cost_table.pop();
            continue;
        }
        let mut stable = build_shadow_compare_payload(
            &diff_keys_for_report,
            &cost_table,
            &reason_table,
            report_truncated,
            report_bytes,
        );
        stable.insert(
            "report_bytes".to_string(),
            Value::Int(max_report_bytes as i64),
        );
        return ShadowCompareBuild {
            payload: stable,
            reason: report_reason,
        };
    }
}

fn build_shadow_compare_payload(
    diff_keys: &[String],
    cost_table: &[Value],
    reason_table: &[Value],
    truncated: bool,
    report_bytes: usize,
) -> BTreeMap<String, Value> {
    let mut divergence = BTreeMap::new();
    for key in diff_keys {
        divergence.insert(key.clone(), Value::Bool(true));
    }

    let mut payload = BTreeMap::new();
    payload.insert(
        "diff_keys".to_string(),
        Value::List(diff_keys.iter().map(|v| Value::String(v.clone())).collect()),
    );
    payload.insert("divergence".to_string(), Value::Map(divergence));
    payload.insert("cost_table".to_string(), Value::List(cost_table.to_vec()));
    payload.insert(
        "reason_table".to_string(),
        Value::List(reason_table.to_vec()),
    );
    payload.insert("truncated".to_string(), Value::Bool(truncated));
    payload.insert("report_bytes".to_string(), Value::Int(report_bytes as i64));
    payload
}

fn collect_shadow_diff_keys(baseline: &Value, branches: &[ShadowBranchView]) -> Vec<String> {
    let mut keys = Vec::new();
    let baseline_map = match baseline {
        Value::Map(map) => Some(map),
        _ => None,
    };
    let mut key_set = BTreeMap::<String, bool>::new();

    if let Some(map) = baseline_map {
        for key in map.keys() {
            key_set.insert(key.clone(), true);
        }
    }
    for branch in branches {
        if let Value::Map(map) = &branch.state_summary {
            for key in map.keys() {
                key_set.insert(key.clone(), true);
            }
        }
    }

    for key in key_set.keys() {
        let baseline_value = baseline_map.and_then(|map| map.get(key));
        let mut diverged = false;
        for branch in branches {
            let branch_value = match &branch.state_summary {
                Value::Map(map) => map.get(key),
                _ => None,
            };
            if branch_value != baseline_value {
                diverged = true;
                break;
            }
        }
        if diverged {
            keys.push(key.clone());
        }
    }
    keys.sort();
    keys
}

#[derive(Debug, Clone, Copy)]
struct Pcg32 {
    state: u64,
    inc: u64,
}

impl Pcg32 {
    fn seeded(init_state: u64, init_seq: u64) -> Self {
        let mut rng = Self {
            state: 0,
            inc: (init_seq << 1) | 1,
        };
        let _ = rng.next_u32();
        rng.state = rng.state.wrapping_add(init_state);
        let _ = rng.next_u32();
        rng
    }

    fn next_u32(&mut self) -> u32 {
        let old_state = self.state;
        self.state = old_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(self.inc);
        let xorshifted = (((old_state >> 18) ^ old_state) >> 27) as u32;
        let rot = (old_state >> 59) as u32;
        xorshifted.rotate_right(rot)
    }
}

fn game_stream_seed(stream: &str, base_seed: u64) -> u64 {
    let mut input = b"pcg32\0".to_vec();
    input.extend_from_slice(stream.as_bytes());
    fnv1a64_bytes(&input) ^ base_seed
}

fn game_tick_seed(tick: i64) -> u64 {
    fnv1a64_bytes(format!("tick|{}", tick.max(0)).as_bytes())
}

fn game_rng_values(stream: &str, tick: i64, count: usize, base_seed: u64) -> Vec<Value> {
    let stream_seed = game_stream_seed(stream, base_seed);
    let init_state = stream_seed ^ game_tick_seed(tick);
    let mut rng = Pcg32::seeded(init_state, stream_seed);
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        let value = i64::from(rng.next_u32() & 0x7fff_ffff);
        out.push(Value::Int(value));
    }
    out
}

fn parse_ui_events(raw: &str) -> Vec<Value> {
    raw.split('|')
        .filter_map(|token| {
            let token = token.trim();
            if token.is_empty() {
                return None;
            }
            Some(parse_ui_event_token(token))
        })
        .collect()
}

fn parse_ui_event_token(token: &str) -> Value {
    let (kind_raw, payload_raw) = token
        .split_once(':')
        .map(|(k, p)| (k.trim(), p.trim()))
        .unwrap_or((token.trim(), ""));
    let kind = kind_raw.to_ascii_lowercase();
    let mut attrs = BTreeMap::new();

    match kind.as_str() {
        "key" => {
            attrs.insert("key".to_string(), Value::String(payload_raw.to_string()));
        }
        "mouse" => {
            let mut parts = payload_raw.split(',');
            let x = parts
                .next()
                .and_then(|v| v.trim().parse::<i64>().ok())
                .unwrap_or(0);
            let y = parts
                .next()
                .and_then(|v| v.trim().parse::<i64>().ok())
                .unwrap_or(0);
            attrs.insert("x".to_string(), Value::Int(x));
            attrs.insert("y".to_string(), Value::Int(y));
        }
        "text" => {
            attrs.insert("text".to_string(), Value::String(payload_raw.to_string()));
        }
        "quit" => {
            attrs.insert("quit".to_string(), Value::Bool(true));
        }
        _ => {
            attrs.insert("raw".to_string(), Value::String(payload_raw.to_string()));
        }
    }

    let mut event = BTreeMap::new();
    event.insert("t".to_string(), Value::String(kind));
    event.insert("a".to_string(), Value::Map(attrs));
    Value::Map(event)
}

fn parse_ui_draw_list(raw: &str) -> Vec<Value> {
    raw.split('|')
        .filter_map(|token| {
            let token = token.trim();
            if token.is_empty() {
                return None;
            }
            let (kind_raw, payload_raw) = token
                .split_once(':')
                .map(|(k, p)| (k.trim(), p.trim()))
                .unwrap_or((token, ""));
            let mut cmd = BTreeMap::new();
            cmd.insert(
                "kind".to_string(),
                Value::String(kind_raw.to_ascii_lowercase()),
            );
            cmd.insert("raw".to_string(), Value::String(payload_raw.to_string()));
            Some(Value::Map(cmd))
        })
        .collect()
}

fn fs_allow_patterns_for_action(action: &str) -> Vec<String> {
    let var_name = match action {
        "read" => "OCL_STD_FS_ALLOW_READ",
        "list" => "OCL_STD_FS_ALLOW_LIST",
        "write" => "OCL_STD_FS_ALLOW_WRITE",
        "remove" => "OCL_STD_FS_ALLOW_REMOVE",
        "rename" => "OCL_STD_FS_ALLOW_RENAME",
        _ => return Vec::new(),
    };
    let Ok(raw) = env::var(var_name) else {
        return Vec::new();
    };
    raw.split([';', ','])
        .filter_map(normalize_fs_pattern)
        .collect()
}

fn normalize_fs_pattern(raw: &str) -> Option<String> {
    let mut pattern = raw.trim().replace('\\', "/");
    while pattern.starts_with("./") {
        pattern = pattern[2..].to_string();
    }
    while pattern.ends_with('/') && pattern.len() > 1 {
        pattern.pop();
    }
    if pattern.is_empty() {
        None
    } else {
        Some(pattern)
    }
}

fn normalize_fs_logical_path(raw: &str) -> Result<String, ReasonCode> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(ReasonCode::FsInvalidPath);
    }
    let input = trimmed.replace('\\', "/");
    if input.contains('\0') {
        return Err(ReasonCode::FsInvalidPath);
    }
    if Path::new(&input).is_absolute() || input.starts_with('/') {
        return Err(ReasonCode::FsPathOutsideSandbox);
    }

    let mut out: Vec<String> = Vec::new();
    for comp in Path::new(&input).components() {
        match comp {
            Component::CurDir => {}
            Component::Normal(seg) => {
                let seg = seg.to_string_lossy();
                if seg.contains(':') {
                    return Err(ReasonCode::FsInvalidPath);
                }
                out.push(seg.to_string());
            }
            Component::ParentDir => return Err(ReasonCode::FsPathOutsideSandbox),
            Component::RootDir | Component::Prefix(_) => {
                return Err(ReasonCode::FsPathOutsideSandbox);
            }
        }
    }
    Ok(out.join("/"))
}

fn fs_match_pattern(pattern: &str, logical_path: &str) -> bool {
    if pattern == "*" || pattern == "**" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix("/**") {
        return logical_path == prefix || logical_path.starts_with(&format!("{prefix}/"));
    }
    logical_path == pattern
}

fn fs_is_allowed(action: &str, logical_path: &str) -> bool {
    let patterns = fs_allow_patterns_for_action(action);
    if patterns.is_empty() {
        return false;
    }
    patterns
        .iter()
        .any(|pattern| fs_match_pattern(pattern, logical_path))
}

fn fs_contains_symlink_components(path: &Path, root: &Path) -> Result<bool, ReasonCode> {
    if !root.exists() {
        return Ok(false);
    }
    let root_abs = root.canonicalize().map_err(|_| ReasonCode::FsIoError)?;
    let path_abs = if path.exists() {
        path.to_path_buf()
    } else {
        path.parent().unwrap_or(path).to_path_buf()
    };
    let mut cursor = root_abs.clone();
    for comp in path_abs.components() {
        if let Component::Normal(seg) = comp {
            cursor.push(seg);
            if !cursor.exists() {
                break;
            }
            let meta = fs::symlink_metadata(&cursor).map_err(|_| ReasonCode::FsIoError)?;
            if meta.file_type().is_symlink() {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn resolve_fs_path(action: &str, raw_path: &str) -> Result<(PathBuf, String), ReasonCode> {
    let logical_path = normalize_fs_logical_path(raw_path)?;
    if !fs_is_allowed(action, &logical_path) {
        return Err(ReasonCode::FsPermissionDenied);
    }
    let root = fs_sandbox_root();
    let full = if logical_path.is_empty() {
        root.clone()
    } else {
        root.join(&logical_path)
    };
    if fs_contains_symlink_components(&full, &root)? {
        return Err(ReasonCode::FsSymlinkDisallowed);
    }
    Ok((full, logical_path))
}

fn fs_write_text_atomic(path: &Path, text: &str) -> Result<(), ReasonCode> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|_| ReasonCode::FsIoError)?;
        }
    }
    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, text.as_bytes()).map_err(|_| ReasonCode::FsIoError)?;
    if path.exists() {
        let meta = fs::metadata(path).map_err(|_| ReasonCode::FsIoError)?;
        if meta.is_dir() {
            return Err(ReasonCode::FsInvalidPath);
        }
        fs::remove_file(path).map_err(|_| ReasonCode::FsIoError)?;
    }
    fs::rename(&tmp_path, path).map_err(|_| ReasonCode::FsIoError)?;
    Ok(())
}

fn to_i64_saturated(value: u64) -> i64 {
    if value > i64::MAX as u64 {
        i64::MAX
    } else {
        value as i64
    }
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
        "std.fs.read_text" => {
            let raw_path = ctx.get("path").cloned().unwrap_or_default();
            let (full_path, _logical_path) = match resolve_fs_path("read", &raw_path) {
                Ok(v) => v,
                Err(reason) => return Result4::insufficient(reason),
            };

            if !full_path.exists() {
                return Result4::insufficient(ReasonCode::FsNotFound);
            }
            let meta = match fs::metadata(&full_path) {
                Ok(v) => v,
                Err(_) => return Result4::insufficient(ReasonCode::FsIoError),
            };
            if meta.is_dir() {
                return Result4::insufficient(ReasonCode::FsInvalidPath);
            }

            let bytes = match fs::read(&full_path) {
                Ok(v) => v,
                Err(_) => return Result4::insufficient(ReasonCode::FsIoError),
            };
            let ctx_max =
                parse_nonnegative_usize(ctx.get("max_bytes")).unwrap_or(fs_max_read_bytes());
            let effective_max = ctx_max.min(fs_max_read_bytes());
            let truncated = bytes.len() > effective_max;
            let used = if truncated {
                &bytes[..effective_max]
            } else {
                bytes.as_slice()
            };
            let text = String::from_utf8_lossy(used).to_string();

            let mut map = BTreeMap::new();
            map.insert("text".to_string(), Value::String(text));
            map.insert("truncated".to_string(), Value::Bool(truncated));
            map.insert("bytes".to_string(), Value::Int(used.len() as i64));
            if truncated {
                Result4::degraded(Value::Map(map), ReasonCode::FsTooLarge)
            } else {
                Result4::ok(Value::Map(map))
            }
        }
        "std.fs.list_dir" => {
            let raw_path = ctx.get("path").cloned().unwrap_or_default();
            let (full_path, _logical_path) = match resolve_fs_path("list", &raw_path) {
                Ok(v) => v,
                Err(reason) => return Result4::insufficient(reason),
            };
            if !full_path.exists() {
                return Result4::insufficient(ReasonCode::FsNotFound);
            }
            if !full_path.is_dir() {
                return Result4::insufficient(ReasonCode::FsInvalidPath);
            }

            let requested_cap =
                parse_nonnegative_usize(ctx.get("cap")).unwrap_or(fs_max_list_entries());
            let effective_cap = requested_cap.min(fs_max_list_entries());
            let mut entries: Vec<Value> = Vec::new();
            let iter = match fs::read_dir(&full_path) {
                Ok(v) => v,
                Err(_) => return Result4::insufficient(ReasonCode::FsIoError),
            };
            for item in iter {
                let item = match item {
                    Ok(v) => v,
                    Err(_) => return Result4::insufficient(ReasonCode::FsIoError),
                };
                let name = item.file_name().to_string_lossy().to_string();
                let meta = match item.metadata() {
                    Ok(v) => v,
                    Err(_) => return Result4::insufficient(ReasonCode::FsIoError),
                };
                let is_dir = meta.is_dir();
                let size = if is_dir {
                    0
                } else {
                    to_i64_saturated(meta.len())
                };
                let mut row = BTreeMap::new();
                row.insert("name".to_string(), Value::String(name));
                row.insert("is_dir".to_string(), Value::Bool(is_dir));
                row.insert("size".to_string(), Value::Int(size));
                entries.push(Value::Map(row));
            }
            entries.sort_by(|a, b| {
                let an = match a {
                    Value::Map(map) => map
                        .get("name")
                        .and_then(Value::as_string)
                        .unwrap_or_default(),
                    _ => String::new(),
                };
                let bn = match b {
                    Value::Map(map) => map
                        .get("name")
                        .and_then(Value::as_string)
                        .unwrap_or_default(),
                    _ => String::new(),
                };
                an.cmp(&bn)
            });

            let truncated = entries.len() > effective_cap;
            if truncated {
                entries.truncate(effective_cap);
            }
            let mut map = BTreeMap::new();
            map.insert("entries".to_string(), Value::List(entries));
            map.insert("truncated".to_string(), Value::Bool(truncated));
            if truncated {
                Result4::degraded(Value::Map(map), ReasonCode::LimitExceeded)
            } else {
                Result4::ok(Value::Map(map))
            }
        }
        "std.fs.stat" => {
            let raw_path = ctx.get("path").cloned().unwrap_or_default();
            let (full_path, _logical_path) = match resolve_fs_path("read", &raw_path) {
                Ok(v) => v,
                Err(reason) => return Result4::insufficient(reason),
            };

            let mut map = BTreeMap::new();
            if !full_path.exists() {
                map.insert("exists".to_string(), Value::Bool(false));
                map.insert("is_dir".to_string(), Value::Bool(false));
                map.insert("size".to_string(), Value::Int(0));
                return Result4::ok(Value::Map(map));
            }
            let meta = match fs::metadata(&full_path) {
                Ok(v) => v,
                Err(_) => return Result4::insufficient(ReasonCode::FsIoError),
            };
            let is_dir = meta.is_dir();
            let size = if is_dir {
                0
            } else {
                to_i64_saturated(meta.len())
            };
            map.insert("exists".to_string(), Value::Bool(true));
            map.insert("is_dir".to_string(), Value::Bool(is_dir));
            map.insert("size".to_string(), Value::Int(size));
            Result4::ok(Value::Map(map))
        }
        "std.fs.write_text" => {
            let path = ctx.get("path").cloned().unwrap_or_default();
            let text = ctx.get("text").cloned().unwrap_or_default();
            let overwrite = parse_bool_ctx(ctx.get("overwrite")).unwrap_or(false);
            let (full_path, _logical_path) = match resolve_fs_path("write", &path) {
                Ok(v) => v,
                Err(reason) => return Result4::insufficient(reason),
            };
            if full_path.exists() && !overwrite {
                return Result4::insufficient(ReasonCode::PolicyDenied);
            }
            if text.len() > fs_max_write_bytes() {
                return Result4::deferred(ReasonCode::LimitExceeded);
            }
            let mut map = BTreeMap::new();
            map.insert("op".to_string(), Value::String("write_text".to_string()));
            map.insert("path".to_string(), Value::String(path.clone()));
            map.insert("text".to_string(), Value::String(text.clone()));
            map.insert("overwrite".to_string(), Value::Bool(overwrite));
            map.insert(
                "pending_write_id".to_string(),
                Value::String(stable_hash64_hex(&format!(
                    "fs_write_text|{path}|{overwrite}|{}",
                    stable_hash64_hex(&text)
                ))),
            );
            Result4::ok(Value::Map(map))
        }
        "std.fs.mkdir" => {
            let path = ctx.get("path").cloned().unwrap_or_default();
            let recursive = parse_bool_ctx(ctx.get("recursive")).unwrap_or(false);
            let (full_path, _logical_path) = match resolve_fs_path("write", &path) {
                Ok(v) => v,
                Err(reason) => return Result4::insufficient(reason),
            };
            if full_path.exists() {
                let meta = match fs::metadata(&full_path) {
                    Ok(v) => v,
                    Err(_) => return Result4::insufficient(ReasonCode::FsIoError),
                };
                if !meta.is_dir() {
                    return Result4::insufficient(ReasonCode::FsInvalidPath);
                }
            }
            let mut map = BTreeMap::new();
            map.insert("op".to_string(), Value::String("mkdir".to_string()));
            map.insert("path".to_string(), Value::String(path.clone()));
            map.insert("recursive".to_string(), Value::Bool(recursive));
            map.insert(
                "pending_write_id".to_string(),
                Value::String(stable_hash64_hex(&format!("fs_mkdir|{path}|{recursive}"))),
            );
            Result4::ok(Value::Map(map))
        }
        "std.fs.remove" => {
            let path = ctx.get("path").cloned().unwrap_or_default();
            let recursive = parse_bool_ctx(ctx.get("recursive")).unwrap_or(false);
            let (full_path, _logical_path) = match resolve_fs_path("remove", &path) {
                Ok(v) => v,
                Err(reason) => return Result4::insufficient(reason),
            };
            if full_path.exists() {
                let meta = match fs::metadata(&full_path) {
                    Ok(v) => v,
                    Err(_) => return Result4::insufficient(ReasonCode::FsIoError),
                };
                if meta.is_dir() && !recursive {
                    return Result4::insufficient(ReasonCode::PolicyDenied);
                }
            }
            let mut map = BTreeMap::new();
            map.insert("op".to_string(), Value::String("remove".to_string()));
            map.insert("path".to_string(), Value::String(path.clone()));
            map.insert("recursive".to_string(), Value::Bool(recursive));
            map.insert(
                "pending_write_id".to_string(),
                Value::String(stable_hash64_hex(&format!("fs_remove|{path}|{recursive}"))),
            );
            Result4::ok(Value::Map(map))
        }
        "std.fs.rename" => {
            let from = ctx.get("from").cloned().unwrap_or_default();
            let to = ctx.get("to").cloned().unwrap_or_default();
            let overwrite = parse_bool_ctx(ctx.get("overwrite")).unwrap_or(false);
            let (from_full, _from_logical) = match resolve_fs_path("rename", &from) {
                Ok(v) => v,
                Err(reason) => return Result4::insufficient(reason),
            };
            let (_to_full, _to_logical) = match resolve_fs_path("rename", &to) {
                Ok(v) => v,
                Err(reason) => return Result4::insufficient(reason),
            };
            if !from_full.exists() {
                return Result4::insufficient(ReasonCode::FsNotFound);
            }
            let mut map = BTreeMap::new();
            map.insert("op".to_string(), Value::String("rename".to_string()));
            map.insert("from".to_string(), Value::String(from.clone()));
            map.insert("to".to_string(), Value::String(to.clone()));
            map.insert("overwrite".to_string(), Value::Bool(overwrite));
            map.insert(
                "pending_write_id".to_string(),
                Value::String(stable_hash64_hex(&format!(
                    "fs_rename|{from}|{to}|{overwrite}"
                ))),
            );
            Result4::ok(Value::Map(map))
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
        "std.kv.get" => {
            let key = ctx.get("key").cloned().unwrap_or_default();
            let store = match load_kv_store() {
                Ok(v) => v,
                Err(reason) => return Result4::insufficient(reason),
            };
            let mut map = BTreeMap::new();
            if let Some(value) = store.get(&key) {
                map.insert("found".to_string(), Value::Bool(true));
                map.insert("value".to_string(), value.clone());
            } else {
                map.insert("found".to_string(), Value::Bool(false));
                map.insert("value".to_string(), Value::Unit);
            }
            Result4::ok(Value::Map(map))
        }
        "std.kv.keys" => {
            let requested_cap = parse_nonnegative_usize(ctx.get("cap")).unwrap_or(100);
            let effective_cap = requested_cap.min(kv_max_keys());
            let store = match load_kv_store() {
                Ok(v) => v,
                Err(reason) => return Result4::insufficient(reason),
            };
            let mut keys: Vec<String> = store.keys().cloned().collect();
            keys.sort();

            let truncated = keys.len() > effective_cap;
            if truncated {
                keys.truncate(effective_cap);
            }
            let mut map = BTreeMap::new();
            map.insert(
                "keys".to_string(),
                Value::List(keys.into_iter().map(Value::String).collect()),
            );
            map.insert("truncated".to_string(), Value::Bool(truncated));
            if truncated {
                Result4::degraded(Value::Map(map), ReasonCode::KvCapExceeded)
            } else {
                Result4::ok(Value::Map(map))
            }
        }
        "std.kv.put" => {
            let key = ctx.get("key").cloned().unwrap_or_default();
            let overwrite = parse_bool_ctx(ctx.get("overwrite")).unwrap_or(false);
            let value = parse_kv_value_from_ctx(&ctx);
            if stringify_json_value(&value).len() > kv_max_value_bytes() {
                return Result4::deferred(ReasonCode::KvCapExceeded);
            }
            let store = match load_kv_store() {
                Ok(v) => v,
                Err(reason) => return Result4::insufficient(reason),
            };
            if !overwrite && store.contains_key(&key) {
                return Result4::insufficient(ReasonCode::PolicyDenied);
            }
            if !store.contains_key(&key) && store.len() >= kv_max_keys() {
                return Result4::deferred(ReasonCode::KvCapExceeded);
            }
            let mut map = BTreeMap::new();
            map.insert("op".to_string(), Value::String("put".to_string()));
            map.insert("key".to_string(), Value::String(key.clone()));
            map.insert("value".to_string(), value);
            map.insert("overwrite".to_string(), Value::Bool(overwrite));
            map.insert(
                "pending_write_id".to_string(),
                Value::String(stable_hash64_hex(&format!("kv_put|{key}|{overwrite}"))),
            );
            Result4::ok(Value::Map(map))
        }
        "std.kv.del" => {
            let key = ctx.get("key").cloned().unwrap_or_default();
            let mut map = BTreeMap::new();
            map.insert("op".to_string(), Value::String("del".to_string()));
            map.insert("key".to_string(), Value::String(key.clone()));
            map.insert(
                "pending_write_id".to_string(),
                Value::String(stable_hash64_hex(&format!("kv_del|{key}"))),
            );
            Result4::ok(Value::Map(map))
        }
        "std.kv.clear" => {
            let prefix = ctx.get("prefix").cloned();
            let mut map = BTreeMap::new();
            map.insert("op".to_string(), Value::String("clear".to_string()));
            if let Some(prefix) = prefix.clone() {
                map.insert("prefix".to_string(), Value::String(prefix));
            }
            map.insert(
                "pending_write_id".to_string(),
                Value::String(stable_hash64_hex(&format!(
                    "kv_clear|{}",
                    prefix.unwrap_or_default()
                ))),
            );
            Result4::ok(Value::Map(map))
        }
        "std.time.now" => {
            let mut map = BTreeMap::new();
            map.insert("unix_ms".to_string(), "1700000000000".to_string());
            Result4::ok(Value::Payload(map))
        }
        "std.time.tick_info" => {
            let mut map = BTreeMap::new();
            let tick =
                parse_nonnegative_i64(ctx.get("tick").or_else(|| ctx.get("ctx_tick"))).unwrap_or(0);
            let dt_ms = parse_nonnegative_i64(ctx.get("dt_ms")).unwrap_or(16).max(1);
            map.insert("tick".to_string(), tick.to_string());
            map.insert("dt_ms".to_string(), dt_ms.to_string());
            Result4::ok(Value::Payload(map))
        }
        "std.time.now_logical" => {
            let mut map = BTreeMap::new();
            let tick =
                parse_nonnegative_i64(ctx.get("tick").or_else(|| ctx.get("ctx_tick"))).unwrap_or(0);
            let dt_ms = parse_nonnegative_i64(ctx.get("dt_ms")).unwrap_or(16).max(1);
            let logical_t = tick.saturating_mul(dt_ms);
            map.insert("t".to_string(), logical_t.to_string());
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
        "engine.ui.run" => {
            let entry_module = match ctx.get("entry_module") {
                Some(v) if !v.trim().is_empty() => v.clone(),
                _ => return Result4::insufficient(ReasonCode::CtxInvalid),
            };
            let tick =
                parse_nonnegative_i64(ctx.get("tick").or_else(|| ctx.get("ctx_tick"))).unwrap_or(0);
            let phase = ctx
                .get("phase")
                .cloned()
                .unwrap_or_else(|| if tick == 0 { "init" } else { "frame" }.to_string());
            let dt_ms = parse_nonnegative_i64(ctx.get("dt_ms"))
                .unwrap_or(game_fixed_dt_ms())
                .max(1);

            let w = parse_nonnegative_i64(ctx.get("w")).unwrap_or(1280).max(1);
            let h = parse_nonnegative_i64(ctx.get("h")).unwrap_or(720).max(1);
            let scale = parse_nonnegative_i64(ctx.get("scale")).unwrap_or(1).max(1);
            let theme = ctx
                .get("theme")
                .cloned()
                .unwrap_or_else(|| "default".to_string());
            let locale = ctx
                .get("locale")
                .cloned()
                .unwrap_or_else(|| "en-US".to_string());

            let requested_input_cap = parse_nonnegative_usize(ctx.get("input_cap"))
                .or_else(|| parse_nonnegative_usize(ctx.get("cap")))
                .unwrap_or(ui_max_input_events());
            let input_cap = requested_input_cap.min(ui_max_input_events());
            let mut input_events =
                parse_ui_events(ctx.get("events").map(String::as_str).unwrap_or(""));
            let input_truncated = input_events.len() > input_cap;
            if input_truncated {
                input_events.truncate(input_cap);
            }

            let requested_draw_cap =
                parse_nonnegative_usize(ctx.get("draw_cap")).unwrap_or(ui_max_draw_cmds());
            let draw_cap = requested_draw_cap.min(ui_max_draw_cmds());
            let mut draw_list =
                parse_ui_draw_list(ctx.get("draw_list").map(String::as_str).unwrap_or(""));
            let draw_truncated = draw_list.len() > draw_cap;
            if draw_truncated {
                draw_list.truncate(draw_cap);
            }

            let mut frame_info = BTreeMap::new();
            frame_info.insert("w".to_string(), Value::Int(w));
            frame_info.insert("h".to_string(), Value::Int(h));
            frame_info.insert("scale".to_string(), Value::Int(scale));
            frame_info.insert("theme".to_string(), Value::String(theme));
            frame_info.insert("locale".to_string(), Value::String(locale));
            frame_info.insert("tick".to_string(), Value::Int(tick));

            let mut payload = BTreeMap::new();
            payload.insert("entry_module".to_string(), Value::String(entry_module));
            payload.insert("phase".to_string(), Value::String(phase));
            payload.insert("tick".to_string(), Value::Int(tick));
            payload.insert("dt_ms".to_string(), Value::Int(dt_ms));
            payload.insert("frame_info".to_string(), Value::Map(frame_info));
            payload.insert("input_events".to_string(), Value::List(input_events));
            payload.insert("input_truncated".to_string(), Value::Bool(input_truncated));
            payload.insert("draw".to_string(), Value::List(draw_list));
            payload.insert("draw_truncated".to_string(), Value::Bool(draw_truncated));
            payload.insert("state".to_string(), parse_engine_state_value(&ctx));
            payload.insert(
                "quit".to_string(),
                Value::Bool(parse_bool_ctx(ctx.get("quit")).unwrap_or(false)),
            );

            if input_truncated
                || draw_truncated
                || requested_input_cap > ui_max_input_events()
                || requested_draw_cap > ui_max_draw_cmds()
            {
                Result4::degraded(Value::Map(payload), ReasonCode::UiCapExceeded)
            } else {
                Result4::ok(Value::Map(payload))
            }
        }
        "engine.game.run" => {
            if !game_enabled() {
                return Result4::insufficient(ReasonCode::GameDisabled);
            }
            let entry_module = match ctx.get("entry_module") {
                Some(v) if !v.trim().is_empty() => v.clone(),
                _ => return Result4::insufficient(ReasonCode::CtxInvalid),
            };
            let tick =
                parse_nonnegative_i64(ctx.get("tick").or_else(|| ctx.get("ctx_tick"))).unwrap_or(0);
            let phase = ctx
                .get("phase")
                .cloned()
                .unwrap_or_else(|| if tick == 0 { "init" } else { "frame" }.to_string());
            let dt_ms = game_fixed_dt_ms().max(1);

            let stream = ctx
                .get("stream")
                .cloned()
                .unwrap_or_else(|| "main".to_string());
            if !game_stream_allowed(&stream) {
                return Result4::insufficient(ReasonCode::GameRngInvalidStream);
            }
            let requested_count = parse_nonnegative_usize(ctx.get("count"))
                .unwrap_or(1)
                .max(1);
            if requested_count > game_rng_max_count() {
                return Result4::deferred(ReasonCode::LimitExceeded);
            }
            let rng_values = game_rng_values(&stream, tick, requested_count, game_base_seed());

            let requested_input_cap = parse_nonnegative_usize(ctx.get("input_cap"))
                .or_else(|| parse_nonnegative_usize(ctx.get("cap")))
                .unwrap_or(ui_max_input_events());
            let input_cap = requested_input_cap.min(ui_max_input_events());
            let mut input_events =
                parse_ui_events(ctx.get("events").map(String::as_str).unwrap_or(""));
            let input_truncated = input_events.len() > input_cap;
            if input_truncated {
                input_events.truncate(input_cap);
            }

            let requested_draw_cap =
                parse_nonnegative_usize(ctx.get("draw_cap")).unwrap_or(ui_max_draw_cmds());
            let draw_cap = requested_draw_cap.min(ui_max_draw_cmds());
            let mut draw_list =
                parse_ui_draw_list(ctx.get("draw_list").map(String::as_str).unwrap_or(""));
            let draw_truncated = draw_list.len() > draw_cap;
            if draw_truncated {
                draw_list.truncate(draw_cap);
            }

            let mut payload = BTreeMap::new();
            payload.insert("entry_module".to_string(), Value::String(entry_module));
            payload.insert("phase".to_string(), Value::String(phase));
            payload.insert("tick".to_string(), Value::Int(tick));
            payload.insert("dt_ms".to_string(), Value::Int(dt_ms));
            payload.insert("stream".to_string(), Value::String(stream));
            payload.insert("count".to_string(), Value::Int(requested_count as i64));
            payload.insert("rng_values".to_string(), Value::List(rng_values));
            payload.insert("state".to_string(), parse_engine_state_value(&ctx));
            payload.insert("input_events".to_string(), Value::List(input_events));
            payload.insert("input_truncated".to_string(), Value::Bool(input_truncated));
            payload.insert("draw".to_string(), Value::List(draw_list));
            payload.insert("draw_truncated".to_string(), Value::Bool(draw_truncated));
            payload.insert(
                "quit".to_string(),
                Value::Bool(parse_bool_ctx(ctx.get("quit")).unwrap_or(false)),
            );

            if input_truncated
                || draw_truncated
                || requested_input_cap > ui_max_input_events()
                || requested_draw_cap > ui_max_draw_cmds()
            {
                Result4::degraded(Value::Map(payload), ReasonCode::UiCapExceeded)
            } else {
                Result4::ok(Value::Map(payload))
            }
        }
        "engine.shadow.preview" => {
            if !shadow_enabled() {
                return Result4::insufficient(ReasonCode::ShadowDisabled);
            }
            let entry_module = match ctx.get("entry_module") {
                Some(v) if !v.trim().is_empty() => v.clone(),
                _ => return Result4::insufficient(ReasonCode::CtxInvalid),
            };
            let disallowed_effects = shadow_disallowed_effect_keys(&ctx);
            if !disallowed_effects.is_empty() {
                return Result4::insufficient(ReasonCode::ShadowEffectDisallowed);
            }
            let variants = match parse_shadow_variants(&ctx) {
                Ok(v) if !v.is_empty() => v,
                _ => return Result4::insufficient(ReasonCode::CtxInvalid),
            };

            let requested_branches = parse_nonnegative_usize(ctx.get("branches"))
                .unwrap_or(variants.len())
                .max(1);
            let requested_step_cap = parse_nonnegative_usize(ctx.get("branch_step_cap"))
                .unwrap_or(shadow_branch_step_cap())
                .max(1);
            let requested_budget_cap = parse_nonnegative_usize(ctx.get("branch_budget_cap"))
                .unwrap_or(shadow_branch_budget_cap())
                .max(1);

            let effective_branches = requested_branches
                .min(variants.len())
                .min(shadow_max_branches());
            let effective_step_cap = requested_step_cap.min(shadow_branch_step_cap());
            let effective_budget_cap = requested_budget_cap.min(shadow_branch_budget_cap());
            let run_truncated = effective_branches < variants.len()
                || requested_branches > shadow_max_branches()
                || requested_step_cap > shadow_branch_step_cap()
                || requested_budget_cap > shadow_branch_budget_cap();
            let tick =
                parse_nonnegative_i64(ctx.get("tick").or_else(|| ctx.get("ctx_tick"))).unwrap_or(0);

            let mut branches = Vec::with_capacity(effective_branches);
            let mut branch_views = Vec::with_capacity(effective_branches);
            let mut signature_join = String::new();
            for (idx, variant) in variants.into_iter().enumerate().take(effective_branches) {
                let variant_json = stringify_json_value(&variant);
                let signature =
                    stable_hash64_hex(&format!("shadow_branch|{idx}|{tick}|{variant_json}"));
                signature_join.push_str(&signature);
                signature_join.push('|');

                let steps =
                    ((fnv1a64_bytes(signature.as_bytes()) % effective_step_cap as u64) + 1) as i64;
                let budget = ((steps as usize)
                    .saturating_mul(16)
                    .min(effective_budget_cap)) as i64;

                let mut cost = BTreeMap::new();
                cost.insert("steps".to_string(), Value::Int(steps));
                cost.insert("budget".to_string(), Value::Int(budget));

                let mut state_summary = BTreeMap::new();
                state_summary.insert("branch_id".to_string(), Value::Int(idx as i64));
                state_summary.insert("variant".to_string(), variant.clone());
                state_summary.insert("tick".to_string(), Value::Int(tick));

                let mut branch = BTreeMap::new();
                branch.insert("id".to_string(), Value::Int(idx as i64));
                branch.insert("outcome".to_string(), Value::String("OK".to_string()));
                branch.insert("signature".to_string(), Value::String(signature.clone()));
                branch.insert("cost".to_string(), Value::Map(cost));
                branch.insert(
                    "state_summary".to_string(),
                    Value::Map(state_summary.clone()),
                );
                branches.push(Value::Map(branch));

                branch_views.push(ShadowBranchView {
                    id: idx as i64,
                    outcome: ResultKind::Ok,
                    reason: None,
                    signature,
                    cost_steps: steps,
                    cost_budget: budget,
                    state_summary: Value::Map(state_summary),
                });
            }

            let branch_digest = stable_hash64_hex(&signature_join);
            let mut run_payload = BTreeMap::new();
            run_payload.insert("branches".to_string(), Value::List(branches));
            run_payload.insert("truncated".to_string(), Value::Bool(run_truncated));
            run_payload.insert(
                "max_branches".to_string(),
                Value::Int(shadow_max_branches() as i64),
            );
            run_payload.insert(
                "branch_step_cap".to_string(),
                Value::Int(effective_step_cap as i64),
            );
            run_payload.insert(
                "branch_budget_cap".to_string(),
                Value::Int(effective_budget_cap as i64),
            );
            run_payload.insert("branch_digest".to_string(), Value::String(branch_digest));

            let baseline_id = parse_nonnegative_i64(ctx.get("baseline_id")).unwrap_or(0);
            let max_diff_keys = parse_nonnegative_usize(ctx.get("max_diff_keys"))
                .unwrap_or(shadow_max_diff_keys())
                .min(shadow_max_diff_keys());
            let max_report_bytes = parse_nonnegative_usize(ctx.get("max_report_bytes"))
                .unwrap_or(shadow_max_report_bytes())
                .min(shadow_max_report_bytes());
            let compare = build_shadow_compare_report(
                &branch_views,
                baseline_id,
                max_diff_keys.max(1),
                max_report_bytes.max(64),
            );

            let mut payload = BTreeMap::new();
            payload.insert("entry_module".to_string(), Value::String(entry_module));
            payload.insert("run".to_string(), Value::Map(run_payload));
            payload.insert("compare".to_string(), Value::Map(compare.payload));

            let reason = compare.reason.or(if run_truncated {
                Some(ReasonCode::ShadowCapExceeded)
            } else {
                None
            });
            if let Some(reason) = reason {
                Result4::degraded(Value::Map(payload), reason)
            } else {
                Result4::ok(Value::Map(payload))
            }
        }
        "std.ui.frame_info" => {
            let mut map = BTreeMap::new();
            let w = parse_nonnegative_i64(ctx.get("w")).unwrap_or(1280).max(1);
            let h = parse_nonnegative_i64(ctx.get("h")).unwrap_or(720).max(1);
            let scale = parse_nonnegative_i64(ctx.get("scale")).unwrap_or(1).max(1);
            let theme = ctx
                .get("theme")
                .cloned()
                .unwrap_or_else(|| "default".to_string());
            let locale = ctx
                .get("locale")
                .cloned()
                .unwrap_or_else(|| "en-US".to_string());
            let tick = parse_nonnegative_i64(ctx.get("ctx_tick").or_else(|| ctx.get("tick")))
                .unwrap_or(0)
                .max(0);
            map.insert("w".to_string(), w.to_string());
            map.insert("h".to_string(), h.to_string());
            map.insert("scale".to_string(), scale.to_string());
            map.insert("theme".to_string(), theme);
            map.insert("locale".to_string(), locale);
            map.insert("ctx_tick".to_string(), tick.to_string());
            map.insert("frame".to_string(), tick.to_string());
            Result4::ok(Value::Payload(map))
        }
        "std.ui.input" => {
            let requested_cap =
                parse_nonnegative_usize(ctx.get("cap")).unwrap_or(ui_max_input_events());
            let effective_cap = requested_cap.min(ui_max_input_events());
            let mut events = parse_ui_events(ctx.get("events").map(String::as_str).unwrap_or(""));
            let truncated = events.len() > effective_cap;
            if truncated {
                events.truncate(effective_cap);
            }
            let mut map = BTreeMap::new();
            map.insert("events".to_string(), Value::List(events));
            map.insert("truncated".to_string(), Value::Bool(truncated));
            if truncated {
                Result4::degraded(Value::Map(map), ReasonCode::UiCapExceeded)
            } else {
                Result4::ok(Value::Map(map))
            }
        }
        "std.ui.draw" => {
            let cap = parse_nonnegative_usize(ctx.get("cap")).unwrap_or(ui_max_draw_cmds());
            if cap > ui_max_draw_cmds() {
                return Result4::deferred(ReasonCode::UiCapExceeded);
            }
            let list_raw = ctx.get("list").cloned().unwrap_or_default();
            let draw_list = parse_ui_draw_list(&list_raw);
            let cmd_count = draw_list.len();
            if cmd_count > cap {
                return Result4::deferred(ReasonCode::UiCapExceeded);
            }

            let mut map = BTreeMap::new();
            map.insert("op".to_string(), Value::String("draw".to_string()));
            map.insert("list".to_string(), Value::List(draw_list));
            map.insert("cmd_count".to_string(), Value::Int(cmd_count as i64));
            map.insert("cap".to_string(), Value::Int(cap as i64));
            map.insert(
                "pending_write_id".to_string(),
                Value::String(stable_hash64_hex(&format!("ui_draw|{cap}|{list_raw}"))),
            );
            Result4::ok(Value::Map(map))
        }
        "std.ui.present" => {
            let mut map = BTreeMap::new();
            map.insert("op".to_string(), Value::String("present".to_string()));
            map.insert(
                "pending_write_id".to_string(),
                Value::String(stable_hash64_hex("ui_present")),
            );
            Result4::ok(Value::Map(map))
        }
        "std.game.tick_info" => {
            if !game_enabled() {
                return Result4::insufficient(ReasonCode::GameDisabled);
            }
            let tick =
                parse_nonnegative_i64(ctx.get("tick").or_else(|| ctx.get("ctx_tick"))).unwrap_or(0);
            let dt_ms = game_fixed_dt_ms().max(1);
            let mut map = BTreeMap::new();
            map.insert("tick".to_string(), Value::Int(tick));
            map.insert("dt_ms".to_string(), Value::Int(dt_ms));
            // Compatibility field kept for older fixtures that still read ctx_tick.
            map.insert("ctx_tick".to_string(), Value::Int(tick));
            Result4::ok(Value::Map(map))
        }
        "std.game.rng" => {
            if !game_enabled() {
                return Result4::insufficient(ReasonCode::GameDisabled);
            }
            let stream = ctx
                .get("stream")
                .cloned()
                .unwrap_or_else(|| "main".to_string());
            if !game_stream_allowed(&stream) {
                return Result4::insufficient(ReasonCode::GameRngInvalidStream);
            }
            let count = ctx
                .get("count")
                .and_then(|v| v.trim().parse::<usize>().ok())
                .unwrap_or(1);
            if count > game_rng_max_count() {
                return Result4::deferred(ReasonCode::LimitExceeded);
            }
            let tick =
                parse_nonnegative_i64(ctx.get("tick").or_else(|| ctx.get("ctx_tick"))).unwrap_or(0);
            let values = game_rng_values(&stream, tick, count, game_base_seed());
            let mut map = BTreeMap::new();
            map.insert("stream".to_string(), Value::String(stream));
            map.insert("count".to_string(), Value::Int(count as i64));
            map.insert("tick".to_string(), Value::Int(tick));
            map.insert("values".to_string(), Value::List(values));
            Result4::ok(Value::Map(map))
        }
        "std.game.state_delta" => {
            if !game_enabled() {
                return Result4::insufficient(ReasonCode::GameDisabled);
            }
            let idempotency_key = match ctx.get("idempotency_key") {
                Some(v) if !v.trim().is_empty() => v.clone(),
                _ => return Result4::insufficient(ReasonCode::CtxInvalid),
            };
            let delta = if let Some(raw_delta) = ctx.get("delta") {
                parse_json_value(raw_delta).unwrap_or_else(|_| Value::String(raw_delta.clone()))
            } else if let Some(raw_delta_json) = ctx.get("delta_json") {
                parse_json_value(raw_delta_json)
                    .unwrap_or_else(|_| Value::String(raw_delta_json.clone()))
            } else {
                return Result4::insufficient(ReasonCode::CtxInvalid);
            };
            let delta_text = stringify_json_value(&delta);
            let delta_bytes = delta_text.len();
            if delta_bytes > game_state_delta_max_bytes() {
                return Result4::deferred(ReasonCode::LimitExceeded);
            }

            let mut map = BTreeMap::new();
            map.insert("op".to_string(), Value::String("state_delta".to_string()));
            map.insert(
                "pending_write_id".to_string(),
                Value::String(stable_hash64_hex(&format!(
                    "game_state_delta|{idempotency_key}|{delta_text}"
                ))),
            );
            map.insert(
                "idempotency_key".to_string(),
                Value::String(idempotency_key),
            );
            map.insert("delta".to_string(), delta);
            map.insert("delta_bytes".to_string(), Value::Int(delta_bytes as i64));
            Result4::ok(Value::Map(map))
        }
        "std.shadow.run" => {
            if !shadow_enabled() {
                return Result4::insufficient(ReasonCode::ShadowDisabled);
            }
            let disallowed_effects = shadow_disallowed_effect_keys(&ctx);
            if !disallowed_effects.is_empty() {
                return Result4::insufficient(ReasonCode::ShadowEffectDisallowed);
            }
            let variants = match parse_shadow_variants(&ctx) {
                Ok(v) if !v.is_empty() => v,
                _ => return Result4::insufficient(ReasonCode::CtxInvalid),
            };

            let requested_branches = parse_nonnegative_usize(ctx.get("branches"))
                .unwrap_or(variants.len())
                .max(1);
            let requested_step_cap = parse_nonnegative_usize(ctx.get("branch_step_cap"))
                .unwrap_or(shadow_branch_step_cap())
                .max(1);
            let requested_budget_cap = parse_nonnegative_usize(ctx.get("branch_budget_cap"))
                .unwrap_or(shadow_branch_budget_cap())
                .max(1);

            let effective_branches = requested_branches
                .min(variants.len())
                .min(shadow_max_branches());
            let effective_step_cap = requested_step_cap.min(shadow_branch_step_cap());
            let effective_budget_cap = requested_budget_cap.min(shadow_branch_budget_cap());
            let truncated = effective_branches < variants.len()
                || requested_branches > shadow_max_branches()
                || requested_step_cap > shadow_branch_step_cap()
                || requested_budget_cap > shadow_branch_budget_cap();
            let tick =
                parse_nonnegative_i64(ctx.get("tick").or_else(|| ctx.get("ctx_tick"))).unwrap_or(0);

            let mut branches = Vec::with_capacity(effective_branches);
            let mut signature_join = String::new();
            for (idx, variant) in variants.into_iter().enumerate().take(effective_branches) {
                let variant_json = stringify_json_value(&variant);
                let signature =
                    stable_hash64_hex(&format!("shadow_branch|{idx}|{tick}|{variant_json}"));
                signature_join.push_str(&signature);
                signature_join.push('|');

                let steps =
                    ((fnv1a64_bytes(signature.as_bytes()) % effective_step_cap as u64) + 1) as i64;
                let budget = ((steps as usize)
                    .saturating_mul(16)
                    .min(effective_budget_cap)) as i64;

                let mut cost = BTreeMap::new();
                cost.insert("steps".to_string(), Value::Int(steps));
                cost.insert("budget".to_string(), Value::Int(budget));

                let mut state_summary = BTreeMap::new();
                state_summary.insert("branch_id".to_string(), Value::Int(idx as i64));
                state_summary.insert("variant".to_string(), variant);
                state_summary.insert("tick".to_string(), Value::Int(tick));

                let mut branch = BTreeMap::new();
                branch.insert("id".to_string(), Value::Int(idx as i64));
                branch.insert("outcome".to_string(), Value::String("OK".to_string()));
                branch.insert("signature".to_string(), Value::String(signature));
                branch.insert("cost".to_string(), Value::Map(cost));
                branch.insert("state_summary".to_string(), Value::Map(state_summary));
                branches.push(Value::Map(branch));
            }

            let branch_digest = stable_hash64_hex(&signature_join);
            let mut payload = BTreeMap::new();
            payload.insert("branches".to_string(), Value::List(branches));
            payload.insert("truncated".to_string(), Value::Bool(truncated));
            payload.insert(
                "max_branches".to_string(),
                Value::Int(shadow_max_branches() as i64),
            );
            payload.insert(
                "branch_step_cap".to_string(),
                Value::Int(effective_step_cap as i64),
            );
            payload.insert(
                "branch_budget_cap".to_string(),
                Value::Int(effective_budget_cap as i64),
            );
            payload.insert("branch_digest".to_string(), Value::String(branch_digest));

            if truncated {
                Result4::degraded(Value::Map(payload), ReasonCode::ShadowCapExceeded)
            } else {
                Result4::ok(Value::Map(payload))
            }
        }
        "std.shadow.compare" => {
            if !shadow_enabled() {
                return Result4::insufficient(ReasonCode::ShadowDisabled);
            }

            let branches = match parse_shadow_compare_branches(&ctx) {
                Ok(v) if !v.is_empty() => v,
                _ => return Result4::insufficient(ReasonCode::CtxInvalid),
            };
            let baseline_id = parse_nonnegative_i64(ctx.get("baseline_id")).unwrap_or(0);
            let max_diff_keys = parse_nonnegative_usize(ctx.get("max_diff_keys"))
                .unwrap_or(shadow_max_diff_keys())
                .min(shadow_max_diff_keys());
            let max_report_bytes = parse_nonnegative_usize(ctx.get("max_report_bytes"))
                .unwrap_or(shadow_max_report_bytes())
                .min(shadow_max_report_bytes());

            let report = build_shadow_compare_report(
                &branches,
                baseline_id,
                max_diff_keys.max(1),
                max_report_bytes.max(64),
            );
            let result_kind = if report.reason.is_some() {
                ResultKind::Degraded
            } else {
                ResultKind::Ok
            };

            if let Some(reason) = report.reason {
                Result4::degraded(Value::Map(report.payload), reason)
            } else if result_kind == ResultKind::Ok {
                Result4::ok(Value::Map(report.payload))
            } else {
                Result4::degraded(Value::Map(report.payload), ReasonCode::ShadowCapExceeded)
            }
        }
        _ => Result4::deferred(ReasonCode::NotImplemented),
    }
}

fn parse_engine_state_value(ctx: &HashMap<String, String>) -> Value {
    if let Some(raw) = ctx.get("state_json") {
        return parse_json_value(raw).unwrap_or_else(|_| Value::String(raw.clone()));
    }
    if let Some(raw) = ctx.get("state") {
        return parse_json_value(raw).unwrap_or_else(|_| Value::String(raw.clone()));
    }
    Value::Map(BTreeMap::new())
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
        "RC-FS-NOT-FOUND" => Some(ReasonCode::FsNotFound),
        "RC-FS-PERMISSION-DENIED" => Some(ReasonCode::FsPermissionDenied),
        "RC-FS-PATH-OUTSIDE-SANDBOX" => Some(ReasonCode::FsPathOutsideSandbox),
        "RC-FS-SYMLINK-DISALLOWED" => Some(ReasonCode::FsSymlinkDisallowed),
        "RC-FS-INVALID-PATH" => Some(ReasonCode::FsInvalidPath),
        "RC-FS-TOO-LARGE" => Some(ReasonCode::FsTooLarge),
        "RC-FS-IO-ERROR" => Some(ReasonCode::FsIoError),
        "RC-KV-NOT-FOUND" => Some(ReasonCode::KvNotFound),
        "RC-KV-PERMISSION-DENIED" => Some(ReasonCode::KvPermissionDenied),
        "RC-KV-CAP-EXCEEDED" => Some(ReasonCode::KvCapExceeded),
        "RC-KV-IO-ERROR" => Some(ReasonCode::KvIoError),
        "RC-TIME-DISABLED" => Some(ReasonCode::TimeDisabled),
        "RC-GAME-DISABLED" => Some(ReasonCode::GameDisabled),
        "RC-GAME-RNG-INVALID-STREAM" => Some(ReasonCode::GameRngInvalidStream),
        "RC-SHADOW-DISABLED" => Some(ReasonCode::ShadowDisabled),
        "RC-SHADOW-CAP-EXCEEDED" => Some(ReasonCode::ShadowCapExceeded),
        "RC-SHADOW-REPORT-TOO-LARGE" => Some(ReasonCode::ShadowReportTooLarge),
        "RC-SHADOW-EFFECT-DISALLOWED" => Some(ReasonCode::ShadowEffectDisallowed),
        "RC-UI-DISABLED" => Some(ReasonCode::UiDisabled),
        "RC-UI-CAP-EXCEEDED" => Some(ReasonCode::UiCapExceeded),
        "RC-LIMIT-EXCEEDED" => Some(ReasonCode::LimitExceeded),
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

fn fnv1a64_bytes(input: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in input {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn stable_hash256_hex(input: &str) -> String {
    let a = stable_hash64_hex(&format!("0|{input}"));
    let b = stable_hash64_hex(&format!("1|{input}"));
    let c = stable_hash64_hex(&format!("2|{input}"));
    let d = stable_hash64_hex(&format!("3|{input}"));
    format!("{a}{b}{c}{d}")
}

fn stable_hash64_hex(input: &str) -> String {
    format!("{:016x}", fnv1a64_bytes(input.as_bytes()))
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

fn parse_nonnegative_i64(raw: Option<&String>) -> Option<i64> {
    let parsed = raw?.trim().parse::<i64>().ok()?;
    if parsed < 0 {
        None
    } else {
        Some(parsed)
    }
}

fn parse_nonnegative_usize(raw: Option<&String>) -> Option<usize> {
    let parsed = raw?.trim().parse::<usize>().ok()?;
    if parsed == 0 {
        None
    } else {
        Some(parsed)
    }
}

fn parse_bool_ctx(raw: Option<&String>) -> Option<bool> {
    match raw?.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" => Some(true),
        "0" | "false" | "no" => Some(false),
        _ => None,
    }
}

fn parse_kv_value_from_ctx(ctx: &HashMap<String, String>) -> Value {
    if let Some(raw_json) = ctx.get("value_json") {
        if let Ok(parsed) = parse_json_value(raw_json) {
            return parsed;
        }
    }
    if let Some(raw_value) = ctx.get("value") {
        return Value::String(raw_value.clone());
    }
    Value::Unit
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
