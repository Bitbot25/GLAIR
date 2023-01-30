use super::Instruction;

struct CtrlFlowEdge {
    from: BlockHandle,
    to: BlockHandle,
}

pub struct CtrlFlow {
    nodes: Vec<Block>,
    edges: Vec<CtrlFlowEdge>,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct BlockHandle {
    index: usize,
}

impl BlockHandle {
    pub fn realise<'a, I>(&self, graph: &'a CtrlFlow) -> &'a Block {
        graph.realise_handle(*self)
    }

    pub fn edges<I>(&self, graph: &CtrlFlow) -> Vec<BlockHandle> {
        graph.edges(*self)
    }
}

impl CtrlFlow {
    pub fn new() -> Self {
        CtrlFlow {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn insert_block(&mut self, block: Block) -> BlockHandle {
        self.nodes.push(block);
        BlockHandle {
            index: self.nodes.len() - 1,
        }
    }

    pub fn realise_handle(&self, handle: BlockHandle) -> &Block {
        &self.nodes[handle.index]
    }

    pub fn realise_handle_mut(&mut self, handle: BlockHandle) -> &mut Block {
        &mut self.nodes[handle.index]
    }

    pub fn edges(&self, handle: BlockHandle) -> Vec<BlockHandle> {
        // TODO: Make this more efficient
        let mut blocks = Vec::new();

        for edge in &self.edges {
            if edge.from == handle {
                blocks.push(edge.to);
            }
        }
        blocks
    }

    pub fn add_directed_edge(&mut self, from: BlockHandle, to: BlockHandle) {
        self.edges.push(CtrlFlowEdge { from, to });
    }
}

pub struct Block {
    pub(crate) ins: Vec<Instruction>,
}

impl Block {
    pub fn new(ins: Vec<Instruction>) -> Self {
        Block { ins }
    }

    pub fn instructions(&self) -> &Vec<Instruction> {
        &self.ins
    }
}
