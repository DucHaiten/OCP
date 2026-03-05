pub mod ast;
pub mod audit;
pub mod budget;
pub mod compile_cache;
pub mod ctx_contract;
pub mod diag;
pub mod exec;
pub mod hir;
pub mod keys;
pub mod lex;
pub mod parse;
pub mod pilot;
pub mod registry;
pub mod result_kind;
pub mod runner;
pub mod schema;
pub mod span;
pub mod typecheck;
pub mod types;
pub mod value;

pub use ast::{Expr, LetPattern, MatchStmt, Program, Stmt};
pub use audit::{TraceEvent, TraceLog};
pub use budget::{BudgetMeter, CommitPolicyMode, ExecConfig, GuardMode};
pub use compile_cache::{
    compile_cache_key_sha256_v1, compile_with_cache, CompileCacheArtifact, CompileCacheError,
    CompileCacheKeyInput, COMPILE_CACHE_HASHER_VERSION_V1,
};
pub use ctx_contract::{
    ConformanceProfile, CtxFieldType, CtxSchemaField, CORE_SEMANTIC_CTX_FIELDS,
};
pub use diag::{DiagPhase, Diagnostic, ErrorCode, ReasonCode};
pub use exec::{
    execute_program, CommitEvent, ExecCacheStats, ExecOutput, Executor, ObserveCacheStats,
};
pub use hir::{
    build_hir, build_hir_and_hash, canonical_hir_bytes, ir_hash_sha256_v1, HirExpr, HirExprKind,
    HirLetPattern, HirProgram, HirPurity, HirStmt, HirStmtKind, HIR_HASHER_VERSION_V1,
    HIR_SCHEMA_VERSION,
};
pub use keys::{parse_key, KeyRef};
pub use lex::{lex, Token, TokenKind};
pub use parse::{parse_program, Parser};
pub use pilot::{
    build_closeout_report, replay_pilot_sequence, run_pilot_sequence, PilotReplayResult,
    PilotSummary, PilotTurnResult, PilotTurnStatus,
};
pub use registry::{
    Cacheability, CapabilityRegistry, DeterminismClass, KeyCapabilityKind, KeyPattern,
    RegistryCheckError,
};
pub use result_kind::{Result4, ResultKind};
pub use runner::{run_fixture_file, run_fixture_source, FixtureRunnerError};
pub use schema::{
    pretty_schema, schema_skeleton, validate_schema_value, FieldConstraints, FieldSpec,
    SchemaIssue, SchemaIssueCode, SchemaType,
};
pub use span::Span;
pub use typecheck::{
    typecheck_program, typecheck_program_with_compat, CompatMode, TypeChecker,
    TypecheckCompatConfig,
};
pub use types::Type;
pub use value::Value;
