pub mod ast;
pub mod audit;
pub mod budget;
pub mod ctx_contract;
pub mod diag;
pub mod exec;
pub mod keys;
pub mod lex;
pub mod parse;
pub mod pilot;
pub mod registry;
pub mod result_kind;
pub mod runner;
pub mod span;
pub mod typecheck;
pub mod types;
pub mod value;

pub use ast::{Expr, MatchStmt, Program, Stmt};
pub use audit::{TraceEvent, TraceLog};
pub use budget::{BudgetMeter, CommitPolicyMode, ExecConfig};
pub use ctx_contract::{
    ConformanceProfile, CtxFieldType, CtxSchemaField, CORE_SEMANTIC_CTX_FIELDS,
};
pub use diag::{DiagPhase, Diagnostic, ErrorCode, ReasonCode};
pub use exec::{execute_program, CommitEvent, ExecOutput, Executor};
pub use keys::{parse_key, KeyRef};
pub use lex::{lex, Token, TokenKind};
pub use parse::{parse_program, Parser};
pub use pilot::{
    build_closeout_report, replay_pilot_sequence, run_pilot_sequence, PilotReplayResult,
    PilotSummary, PilotTurnResult, PilotTurnStatus,
};
pub use registry::{CapabilityRegistry, KeyPattern, RegistryCheckError};
pub use result_kind::{Result4, ResultKind};
pub use runner::{run_fixture_file, run_fixture_source, FixtureRunnerError};
pub use span::Span;
pub use typecheck::{typecheck_program, TypeChecker};
pub use types::Type;
pub use value::Value;
