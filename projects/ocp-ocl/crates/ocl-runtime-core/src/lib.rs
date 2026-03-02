use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;

use ocp_ocl::ocp_ocl as engine;

pub use engine::{
    parse_program, typecheck_program, DiagPhase, Diagnostic, ExecConfig, ExecOutput, ReasonCode,
    Span, Type,
};

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
