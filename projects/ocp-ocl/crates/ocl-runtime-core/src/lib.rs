use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;

use ocp_ocl::ocp_ocl as engine;

mod bytecode;
mod ir;
mod source_map;
mod vm;

pub use bytecode::{assemble as assemble_bytecode, BytecodeOp, BytecodeProgram};
pub use engine::{
    parse_program, typecheck_program, DiagPhase, Diagnostic, ErrorCode, ExecConfig, ExecOutput,
    Expr, Program, ReasonCode, ResultKind, Span, Stmt, TraceEvent, Type,
};
pub use ir::{lower_program as lower_to_ir, IrOp, IrOpKind, TypedIrProgram};
pub use source_map::{build_source_map, SourceMap, SourceMapEntry};
pub use vm::{execute_compiled as execute_vm, VmProgram};

#[derive(Debug)]
pub enum RuntimeCoreError {
    Io(std::io::Error),
    Diagnostic(Diagnostic),
}

impl Display for RuntimeCoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "IO error: {err}"),
            Self::Diagnostic(diag) => {
                write!(f, "{}: {}", diag.code.as_str(), diag.message)
            }
        }
    }
}

impl std::error::Error for RuntimeCoreError {}

impl From<std::io::Error> for RuntimeCoreError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<Diagnostic> for RuntimeCoreError {
    fn from(value: Diagnostic) -> Self {
        Self::Diagnostic(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunEngine {
    Interpreter,
    Bytecode,
    Dual,
}

impl RunEngine {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Interpreter => "interpreter",
            Self::Bytecode => "bytecode",
            Self::Dual => "dual",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledProgram {
    pub program: Program,
    pub ir: TypedIrProgram,
    pub bytecode: BytecodeProgram,
    pub source_map: SourceMap,
}

pub fn normalize_text(input: &str) -> String {
    let normalized = input.replace("\r\n", "\n").replace('\r', "\n");
    let mut lines: Vec<&str> = normalized.split('\n').collect();
    while matches!(lines.last(), Some(last) if last.is_empty()) {
        lines.pop();
    }
    let mut out = String::new();
    for line in lines {
        out.push_str(line.trim_end_matches([' ', '\t']));
        out.push('\n');
    }
    if out.is_empty() {
        out.push('\n');
    }
    out
}

pub fn check_source(source: &str, file_id: u32) -> Result<(), Diagnostic> {
    let program = parse_program(source, file_id)?;
    typecheck_program(&program)
}

pub fn run_source(source: &str, file_id: u32, step_cap: u32) -> Result<ExecOutput, Diagnostic> {
    let program = parse_program(source, file_id)?;
    typecheck_program(&program)?;
    engine::execute_program(&program, ExecConfig { step_cap })
}

pub fn compile_source(source: &str, file_id: u32) -> Result<CompiledProgram, Diagnostic> {
    let program = parse_program(source, file_id)?;
    typecheck_program(&program)?;
    let ir = lower_to_ir(&program);
    let bytecode = assemble_bytecode(&ir);
    let source_map = build_source_map(&ir);
    Ok(CompiledProgram {
        program,
        ir,
        bytecode,
        source_map,
    })
}

pub fn run_compiled(compiled: &CompiledProgram, step_cap: u32) -> Result<ExecOutput, Diagnostic> {
    let vm_program = VmProgram {
        bytecode: compiled.bytecode.clone(),
        source_map: compiled.source_map.clone(),
    };
    execute_vm(&compiled.program, &vm_program, step_cap)
}

pub fn run_source_with_engine(
    source: &str,
    file_id: u32,
    step_cap: u32,
    run_engine: RunEngine,
) -> Result<ExecOutput, Diagnostic> {
    match run_engine {
        RunEngine::Interpreter => run_source(source, file_id, step_cap),
        RunEngine::Bytecode => {
            let compiled = compile_source(source, file_id)?;
            run_compiled(&compiled, step_cap)
        }
        RunEngine::Dual => {
            let interpreted = run_source(source, file_id, step_cap)?;
            let compiled = compile_source(source, file_id)?;
            let bytecode = run_compiled(&compiled, step_cap)?;
            if interpreted.signature != bytecode.signature {
                return Err(Diagnostic::new(
                    ErrorCode::ECapabilityDenied,
                    DiagPhase::Runtime,
                    Span::new(file_id, 0, 0, 1, 1),
                    format!(
                        "V-DUAL-PARITY-MISMATCH: interpreter={} bytecode={}",
                        interpreted.signature, bytecode.signature
                    ),
                ));
            }
            Ok(bytecode)
        }
    }
}

pub fn check_file(path: &Path, file_id: u32) -> Result<(), RuntimeCoreError> {
    let source = fs::read_to_string(path)?;
    check_source(&source, file_id)?;
    Ok(())
}

pub fn run_file(path: &Path, file_id: u32, step_cap: u32) -> Result<ExecOutput, RuntimeCoreError> {
    let source = fs::read_to_string(path)?;
    let out = run_source(&source, file_id, step_cap)?;
    Ok(out)
}

pub fn compile_file(path: &Path, file_id: u32) -> Result<CompiledProgram, RuntimeCoreError> {
    let source = fs::read_to_string(path)?;
    let compiled = compile_source(&source, file_id)?;
    Ok(compiled)
}

pub fn run_file_with_engine(
    path: &Path,
    file_id: u32,
    step_cap: u32,
    run_engine: RunEngine,
) -> Result<ExecOutput, RuntimeCoreError> {
    let source = fs::read_to_string(path)?;
    let out = run_source_with_engine(&source, file_id, step_cap, run_engine)?;
    Ok(out)
}
