use crate::ir::{IrOpKind, TypedIrProgram};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BytecodeOp {
    ModuleDecl,
    ImportDecl,
    ExportDecl,
    StructDecl,
    EnumDecl,
    FnDef,
    Let,
    TryLet,
    Guard,
    Return,
    Repeat,
    ForEachCap,
    ForRange,
    Observe,
    Commit,
    Condition,
    Entangle,
    Match,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BytecodeProgram {
    pub ops: Vec<BytecodeOp>,
}

impl BytecodeProgram {
    pub fn op_count(&self) -> usize {
        self.ops.len()
    }
}

pub fn assemble(ir: &TypedIrProgram) -> BytecodeProgram {
    let ops = ir
        .ops
        .iter()
        .map(|op| match op.kind {
            IrOpKind::ModuleDecl => BytecodeOp::ModuleDecl,
            IrOpKind::ImportDecl => BytecodeOp::ImportDecl,
            IrOpKind::ExportDecl => BytecodeOp::ExportDecl,
            IrOpKind::StructDecl => BytecodeOp::StructDecl,
            IrOpKind::EnumDecl => BytecodeOp::EnumDecl,
            IrOpKind::FnDef => BytecodeOp::FnDef,
            IrOpKind::Let => BytecodeOp::Let,
            IrOpKind::TryLet => BytecodeOp::TryLet,
            IrOpKind::Guard => BytecodeOp::Guard,
            IrOpKind::Return => BytecodeOp::Return,
            IrOpKind::Repeat => BytecodeOp::Repeat,
            IrOpKind::ForEachCap => BytecodeOp::ForEachCap,
            IrOpKind::ForRange => BytecodeOp::ForRange,
            IrOpKind::Observe => BytecodeOp::Observe,
            IrOpKind::Commit => BytecodeOp::Commit,
            IrOpKind::Condition => BytecodeOp::Condition,
            IrOpKind::Entangle => BytecodeOp::Entangle,
            IrOpKind::Match => BytecodeOp::Match,
        })
        .collect();
    BytecodeProgram { ops }
}
