use std::fs;
use std::path::Path;

use crate::ocp_ocl::{
    execute_program, parse_program, typecheck_program, Diagnostic, ExecConfig, ExecOutput,
};

#[derive(Debug, Clone)]
pub enum FixtureRunnerError {
    Io(String),
    Diag(Diagnostic),
}

impl From<Diagnostic> for FixtureRunnerError {
    fn from(value: Diagnostic) -> Self {
        Self::Diag(value)
    }
}

pub fn run_fixture_source(
    source: &str,
    file_id: u32,
    config: ExecConfig,
) -> Result<ExecOutput, FixtureRunnerError> {
    let program = parse_program(source, file_id)?;
    typecheck_program(&program)?;
    let out = execute_program(&program, config)?;
    Ok(out)
}

pub fn run_fixture_file(
    path: &Path,
    file_id: u32,
    config: ExecConfig,
) -> Result<ExecOutput, FixtureRunnerError> {
    let src = fs::read_to_string(path).map_err(|e| {
        FixtureRunnerError::Io(format!("failed to read fixture '{}': {e}", path.display()))
    })?;
    run_fixture_source(&src, file_id, config)
}
