use ocp::ocp::{execute_program, Diagnostic, ExecConfig, ExecOutput, Program};

use crate::{bytecode::BytecodeProgram, source_map::SourceMap};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VmProgram {
    pub bytecode: BytecodeProgram,
    pub source_map: SourceMap,
}

pub fn execute_compiled(
    program: &Program,
    _vm_program: &VmProgram,
    config: ExecConfig,
) -> Result<ExecOutput, Diagnostic> {
    execute_program(program, config)
}
