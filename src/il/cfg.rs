use super::Instruction;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Location {
    offset: usize,
    block: BlockHandle,
}

impl Location {
    pub fn new(block: BlockHandle, offset: usize) -> Self {
        Location { offset, block }
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn block_handle(&self) -> BlockHandle {
        self.block
    }

    pub fn is_after(&self, graph: &CtrlFlow, other: &Location) -> bool {
        if self.block == other.block {
            self.offset > other.offset
        } else {
            graph.has_forward_path(other.block, self.block)
        }
    }

    pub fn is_before(&self, graph: &CtrlFlow, other: &Location) -> bool {
        if self.block == other.block {
            self.offset < other.offset
        } else {
            graph.has_backwards_path(self.block, other.block)
        }
    }
}

struct CtrlFlowEdge {
    from: BlockHandle,
    to: BlockHandle,
}

pub struct CtrlFlow {
    nodes: Vec<Block>,
    edges: Vec<CtrlFlowEdge>,
}

#[derive(Hash, Debug, PartialEq, Eq, Copy, Clone)]
pub struct BlockHandle {
    index: usize,
}

impl BlockHandle {
    pub fn realise<'a>(&self, graph: &'a CtrlFlow) -> &'a Block {
        graph.realise_handle(*self)
    }

    pub fn descendants(&self, graph: &CtrlFlow) -> Vec<BlockHandle> {
        graph.descendants(*self)
    }

    pub fn predecessors(&self, graph: &CtrlFlow) -> Vec<BlockHandle> {
        graph.predecessors(*self)
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

    pub fn predecessors(&self, handle: BlockHandle) -> Vec<BlockHandle> {
        // TODO: Make this more efficient
        let mut blocks = Vec::new();

        for edge in &self.edges {
            if edge.to == handle {
                blocks.push(edge.from);
            }
        }
        blocks
    }

    pub fn descendants(&self, handle: BlockHandle) -> Vec<BlockHandle> {
        // TODO: Make this more efficient
        let mut blocks = Vec::new();

        for edge in &self.edges {
            if edge.from == handle {
                blocks.push(edge.to);
            }
        }
        blocks
    }

    pub fn has_forward_path(&self, from: BlockHandle, to: BlockHandle) -> bool {
        let from_direct_neighbors = self.descendants(from);
        let mut stack = from_direct_neighbors;
        while let Some(h) = stack.pop() {
            if h == to {
                return true;
            }
            // We've gone in a circle
            if h == from {
                return false;
            }

            stack.extend_from_slice(&*self.descendants(h));
        }
        return false;
    }

    pub fn has_backwards_path(&self, from: BlockHandle, to: BlockHandle) -> bool {
        let from_direct_neighbors = self.predecessors(from);
        let mut stack = from_direct_neighbors;
        while let Some(h) = stack.pop() {
            if h == to {
                return true;
            }
            // We've gone in a circle
            if h == from {
                return false;
            }

            stack.extend_from_slice(&*self.predecessors(h));
        }
        return false;
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
