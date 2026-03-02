pub mod diag;
pub mod lex;
pub mod parse;
pub mod span;
pub mod ast;
pub mod typecheck;
pub mod types;
pub mod result_kind;
pub mod value;
pub mod budget;
pub mod exec;
pub mod keys;
pub mod registry;
pub mod audit;
pub mod runner;
pub mod pilot;

pub use diag::{DiagPhase, Diagnostic, ErrorCode, ReasonCode};
pub use lex::{lex, Token, TokenKind};
pub use parse::{parse_program, Parser};
pub use span::Span;
pub use ast::{Expr, MatchStmt, Program, Stmt};
pub use typecheck::{typecheck_program, TypeChecker};
pub use types::Type;
pub use result_kind::{Result4, ResultKind};
pub use value::Value;
pub use budget::{BudgetMeter, ExecConfig};
pub use exec::{execute_program, CommitEvent, ExecOutput, Executor};
pub use keys::{parse_key, KeyRef};
pub use registry::{CapabilityRegistry, KeyPattern, RegistryCheckError};
pub use audit::{TraceEvent, TraceLog};
pub use runner::{run_fixture_file, run_fixture_source, FixtureRunnerError};
pub use pilot::{
    build_closeout_report, replay_pilot_sequence, run_pilot_sequence, PilotReplayResult,
    PilotTurnResult, PilotTurnStatus, PilotSummary,
};
