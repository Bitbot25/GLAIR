pub struct ControlFlowGraph<'this, I> {
    pub blocks: Vec<Block<'this, I>>,
}

pub struct Block<'cfg, I> {
    pub intrs: Vec<I>,
    pub exit: &'cfg Block<'cfg, I>,
}
