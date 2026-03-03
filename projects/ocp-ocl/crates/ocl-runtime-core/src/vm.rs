use ocp_ocl::ocp_ocl::{execute_program, Diagnostic, ExecConfig, ExecOutput, Program};

use crate::{bytecode::BytecodeProgram, source_map::SourceMap};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VmProgram {
    pub bytecode: BytecodeProgram,
    pub source_map: SourceMap,
}

pub fn execute_compiled(
    program: &Program,
    _vm_program: &VmProgram,
    step_cap: u32,
) -> Result<ExecOutput, Diagnostic> {
    execute_program(program, ExecConfig { step_cap })
}
