use ocp_ocl::ocp_ocl::Span;

use crate::ir::TypedIrProgram;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceMapEntry {
    pub pc: usize,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SourceMap {
    pub entries: Vec<SourceMapEntry>,
}

impl SourceMap {
    pub fn span_for_pc(&self, pc: usize) -> Option<Span> {
        self.entries
            .iter()
            .find_map(|entry| (entry.pc == pc).then_some(entry.span))
    }
}

pub fn build_source_map(ir: &TypedIrProgram) -> SourceMap {
    let entries = ir
        .ops
        .iter()
        .enumerate()
        .map(|(pc, op)| SourceMapEntry { pc, span: op.span })
        .collect();
    SourceMap { entries }
}
