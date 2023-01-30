use galloc::{
    cfg::{GAllocBlock, GAllocControlFlowGraph, GAllocInstruction, GAllocVariable},
    TypeClass,
};

use super::{
    cfg::{Block, BlockHandle, CtrlFlow},
    Instruction, SSARegister,
};

impl GAllocVariable for SSARegister {
    fn type_class(&self) -> TypeClass {
        todo!()
    }
}

impl GAllocInstruction for Instruction {
    type Variable = SSARegister;

    fn variables_read_by_instruction(&self) -> Vec<&SSARegister> {
        todo!()
    }
}

impl GAllocBlock for Block {
    type Instruction = Instruction;
    type InstructionList = Vec<Instruction>;
    type InstructionIter<'a> = std::slice::Iter<'a, Instruction>;
    type Handle = BlockHandle;

    fn instructions(&self) -> &Self::InstructionList {
        // Note that this calls the GLAIR implementation, not galloc
        self.instructions()
    }

    fn instructions_iter<'a>(&'a self) -> Self::InstructionIter<'a> {
        // Note that this calls the GLAIR implementation, not galloc
        self.instructions().iter()
    }
}

impl GAllocControlFlowGraph for CtrlFlow {
    type Block = Block;
    type EdgesIterator = std::vec::IntoIter<BlockHandle>;

    fn block_get(&self, handle: BlockHandle) -> &Block {
        self.realise_handle(handle)
    }

    fn block_edges(&self, handle: BlockHandle) -> Self::EdgesIterator {
        self.edges(handle).into_iter()
    }
}
