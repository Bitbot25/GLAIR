use crate::ins::Instruction;
use gcf_bb as bb;

pub type BasicBlock = bb::BasicBlock<Instruction>;

pub struct Function {
    name: String,
    entry_block: BasicBlock,
}
