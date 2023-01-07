#[derive(Debug)]
pub struct ControlFlowGraph<'this, I> {
    pub blocks: Vec<Block<'this, I>>,
}

#[derive(Debug)]
pub struct Block<'cfg, I> {
    pub id: usize,
    pub intrs: Vec<I>,
    pub exit: &'cfg Block<'cfg, I>,
}
