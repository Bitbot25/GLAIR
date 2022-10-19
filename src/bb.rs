use crate::ssa;

#[derive(Debug)]
pub struct BasicBlock {
    pub ins_list: Vec<ssa::Ins>,
    pub terminator: Terminator,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum Terminator {
    Ret(ssa::Variable),
    Jmp(Box<BasicBlock>),
    Void,
}
