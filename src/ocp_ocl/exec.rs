use std::collections::{BTreeMap, HashMap};

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

pub struct Executor {
    env: HashMap<String, Value>,
    meter: BudgetMeter,
    next_origin_id: u64,
    registry: CapabilityRegistry,
    observations: HashMap<u64, ObservationMeta>,
    commits: Vec<CommitEvent>,
    trace: TraceLog,
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
        }
    }

    pub fn run(mut self, program: &Program) -> Result<ExecOutput, Diagnostic> {
        for stmt in &program.statements {
            self.exec_stmt(stmt)?;
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

    fn exec_block(&mut self, block: &[Stmt]) -> Result<(), Diagnostic> {
        for stmt in block {
            self.exec_stmt(stmt)?;
        }
        Ok(())
    }

    fn exec_stmt(&mut self, stmt: &Stmt) -> Result<(), Diagnostic> {
        self.tick(stmt_span(stmt))?;
        match stmt {
            Stmt::Let { name, value, .. } => {
                let v = self.eval_expr(value)?;
                self.env.insert(name.clone(), v);
                Ok(())
            }
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
                        ErrorCode::ECapabilityDenied,
                        DiagPhase::Exec,
                        key.span(),
                        "observe key must evaluate to string",
                    )
                })?;
                let tier_lit = as_string_runtime(&tier_value).ok_or_else(|| {
                    Diagnostic::new(
                        ErrorCode::ECapabilityDenied,
                        DiagPhase::Exec,
                        tier.span(),
                        "observe tier must evaluate to string",
                    )
                })?;
                let ctx_lit = as_ctx_runtime(&ctx_value).ok_or_else(|| {
                    Diagnostic::new(
                        ErrorCode::ECapabilityDenied,
                        DiagPhase::Exec,
                        ctx.span(),
                        "observe ctx must evaluate to ctx(...) value",
                    )
                })?;
                let budget_units = as_budget_runtime(&budget_value).ok_or_else(|| {
                    Diagnostic::new(
                        ErrorCode::ECapabilityDenied,
                        DiagPhase::Exec,
                        budget.span(),
                        "observe budget must evaluate to budget(...) value",
                    )
                })?;

                self.trace.push(TraceEvent::ObserveStart {
                    key: key_lit.to_string(),
                    tier: tier_lit.to_string(),
                    ctx: ctx_lit.to_string(),
                    budget: budget_units,
                });

                let origin_id = self.allocate_origin_id();
                let r = match self.registry.check_observe(key_lit, ctx_lit) {
                    Ok(kref) => observe_stub_result(&kref.raw),
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
                Ok(())
            }
            Stmt::Commit { value, span } => self.exec_commit(value, *span),
            Stmt::Condition { value, span } => self.exec_condition(value, *span),
            Stmt::Match(m) => self.exec_match(m),
        }
    }

    fn exec_match(&mut self, m: &MatchStmt) -> Result<(), Diagnostic> {
        let v = self.eval_expr(&m.value)?;
        let Value::Result4(r) = v else {
            return Err(Diagnostic::new(
                ErrorCode::ECommitForbidden,
                DiagPhase::Exec,
                m.span,
                "match expects Result4 value at runtime",
            ));
        };

        self.trace.push(TraceEvent::MatchArmSelected { arm: r.kind });
        match r.kind {
            ResultKind::Ok => self.exec_block(&m.ok_arm),
            ResultKind::Degraded => self.exec_block(&m.degraded_arm),
            ResultKind::Insufficient => self.exec_block(&m.insufficient_arm),
            ResultKind::Deferred => self.exec_block(&m.deferred_arm),
        }
    }

    fn exec_condition(&mut self, expr: &Expr, span: Span) -> Result<(), Diagnostic> {
        let v = self.eval_expr(expr)?;
        match v {
            Value::Bool(true) => {
                self.trace.push(TraceEvent::ConditionCheck { value: true });
                Ok(())
            }
            Value::Bool(false) => {
                self.trace.push(TraceEvent::ConditionCheck { value: false });
                Err(Diagnostic::new(
                    ErrorCode::EConditionFalse,
                    DiagPhase::Exec,
                    span,
                    "condition(...) evaluated to false",
                ))
            }
            _ => Err(Diagnostic::new(
                ErrorCode::EConditionFalse,
                DiagPhase::Exec,
                span,
                "condition(...) expects bool at runtime",
            )),
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
                ErrorCode::ECommitForbidden,
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
                ErrorCode::ECommitForbidden,
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
                ErrorCode::ECommitForbidden,
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
                        ErrorCode::ECommitForbidden,
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
                    ErrorCode::ECommitForbidden,
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
                    ErrorCode::ECommitForbidden,
                    DiagPhase::Exec,
                    *span,
                    format!("unknown runtime identifier `{name}`"),
                )
            }),
            Expr::Call { callee, args, span } => self.eval_call(callee, args, *span),
            Expr::FieldAccess { base, field, span } => {
                let b = self.eval_expr(base)?;
                match b {
                    Value::Payload(map) => Ok(map
                        .get(field)
                        .map(|v| Value::String(v.clone()))
                        .unwrap_or(Value::Unknown)),
                    Value::Result4(r) => match r.payload {
                        Some(Value::Payload(map)) => Ok(map
                            .get(field)
                            .map(|v| Value::String(v.clone()))
                            .unwrap_or(Value::Unknown)),
                        _ => Ok(Value::Unknown),
                    },
                    _ => Err(Diagnostic::new(
                        ErrorCode::ECommitForbidden,
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
                        ErrorCode::ECommitForbidden,
                        DiagPhase::Exec,
                        span,
                        "budget(...) expects 1 argument",
                    ));
                }
                let arg = self.eval_expr(&args[0])?;
                match arg {
                    Value::Int(v) if v >= 0 => Ok(Value::Budget(v as u32)),
                    _ => Err(Diagnostic::new(
                        ErrorCode::ECommitForbidden,
                        DiagPhase::Exec,
                        span,
                        "budget(...) expects non-negative int",
                    )),
                }
            }
            "ctx" => {
                if args.len() != 1 {
                    return Err(Diagnostic::new(
                        ErrorCode::ECommitForbidden,
                        DiagPhase::Exec,
                        span,
                        "ctx(...) expects 1 argument",
                    ));
                }
                let arg = self.eval_expr(&args[0])?;
                match arg {
                    Value::String(v) => Ok(Value::Ctx(v)),
                    _ => Err(Diagnostic::new(
                        ErrorCode::ECommitForbidden,
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
                    ErrorCode::ECommitForbidden,
                    DiagPhase::Exec,
                    span,
                    "payload() in V1-D accepts 0 args only",
                ))
            }
            _ => Err(Diagnostic::new(
                ErrorCode::ECommitForbidden,
                DiagPhase::Exec,
                span,
                format!("unknown runtime call `{callee}`"),
            )),
        }
    }

    fn tick(&mut self, span: Span) -> Result<(), Diagnostic> {
        self.meter.tick().map_err(|_| {
            Diagnostic::new(
                ErrorCode::EBudgetExceeded,
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
        Stmt::Let { span, .. }
        | Stmt::Observe { span, .. }
        | Stmt::Commit { span, .. }
        | Stmt::Condition { span, .. } => *span,
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

fn observe_stub_result(key: &str) -> Result4<Value> {
    if key.starts_with("world.ok") {
        return Result4::ok(stub_payload(key));
    }
    if key.starts_with("world.degraded") {
        return Result4::degraded(stub_payload(key), ReasonCode::AdapterFailed);
    }
    Result4::deferred(ReasonCode::NotImplemented)
}

fn stub_payload(key: &str) -> Value {
    let mut map = BTreeMap::new();
    map.insert("key".to_string(), key.to_string());
    map.insert("status".to_string(), "sample".to_string());
    Value::Payload(map)
}
