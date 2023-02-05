use super::Instruction;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Location {
    offset: usize,
    block: BlockHandle,
}

enum LocationExpandMode<T> {
    Single(Option<Location>),
    Boundary(T),
}

pub struct LocationExpandRight<'a> {
    mode: LocationExpandMode<Descendants<'a>>,
}

impl<'a> Iterator for LocationExpandRight<'a> {
    type Item = Location;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.mode {
            LocationExpandMode::Single(val) => val.take(),
            LocationExpandMode::Boundary(iter) => {
                iter.next().map(|handle| Location::new(handle, 0))
            }
        }
    }
}

pub struct LocationExpandLeft<'a> {
    cfg: &'a CtrlFlow,
    mode: LocationExpandMode<Predecessors<'a>>,
}

impl<'a> Iterator for LocationExpandLeft<'a> {
    type Item = Location;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.mode {
            LocationExpandMode::Single(val) => val.take(),
            LocationExpandMode::Boundary(iter) => iter.next().map(|handle| {
                Location::new(
                    handle,
                    self.cfg.realise_handle(handle).instructions().len() - 1,
                )
            }),
        }
    }
}

impl Location {
    pub fn new(block: BlockHandle, offset: usize) -> Self {
        Location { offset, block }
    }

    pub fn expanded_right<'a>(&'a self, cfg: &'a CtrlFlow) -> LocationExpandRight {
        LocationExpandRight {
            mode: if self.touches_max_in_block(cfg) {
                LocationExpandMode::Boundary(cfg.descendants(self.block))
            } else {
                LocationExpandMode::Single(Some(Location::new(self.block, self.offset + 1)))
            },
        }
    }

    pub fn expanded_left<'a>(&'a self, cfg: &'a CtrlFlow) -> LocationExpandLeft {
        LocationExpandLeft {
            mode: if self.touches_min_in_block() {
                LocationExpandMode::Boundary(cfg.predecessors(self.block))
            } else {
                LocationExpandMode::Single(Some(Location::new(self.block, self.offset - 1)))
            },
            cfg,
        }
    }

    #[inline]
    pub fn touches_max_in_block(&self, cfg: &CtrlFlow) -> bool {
        let ins_len = cfg.realise_handle(self.block).instructions().len();
        self.offset >= ins_len - 1
    }

    #[inline]
    pub fn touches_min_in_block(&self) -> bool {
        self.offset == 0
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

#[derive(Debug)]
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

    pub fn predecessors(&self, handle: BlockHandle) -> Predecessors {
        Predecessors::new(handle, &self.edges)
    }

    pub fn descendants(&self, handle: BlockHandle) -> Descendants {
        Descendants::new(handle, &self.edges)
    }

    pub fn has_forward_path(&self, from: BlockHandle, to: BlockHandle) -> bool {
        let mut stack: Vec<BlockHandle> = self.descendants(from).collect();
        while let Some(h) = stack.pop() {
            if h == to {
                return true;
            }
            // We've gone in a circle
            if h == from {
                return false;
            }

            stack.extend_from_slice(&self.descendants(h).collect::<Vec<BlockHandle>>());
        }
        false
    }

    pub fn has_backwards_path(&self, from: BlockHandle, to: BlockHandle) -> bool {
        let mut stack: Vec<BlockHandle> = self.predecessors(from).collect();
        while let Some(h) = stack.pop() {
            if h == to {
                return true;
            }
            // We've gone in a circle
            if h == from {
                return false;
            }

            stack.extend_from_slice(&self.predecessors(h).collect::<Vec<BlockHandle>>());
        }
        false
    }

    pub fn add_directed_edge(&mut self, from: BlockHandle, to: BlockHandle) {
        self.edges.push(CtrlFlowEdge { from, to });
    }
}

pub struct Descendants<'a> {
    handle: BlockHandle,
    edges: &'a Vec<CtrlFlowEdge>,
    index: usize,
}

impl<'a> Descendants<'a> {
    fn new(handle: BlockHandle, edges: &'a Vec<CtrlFlowEdge>) -> Self {
        Self {
            handle,
            edges,
            index: 0,
        }
    }
}

impl<'a> Iterator for Descendants<'a> {
    type Item = BlockHandle;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.edges.len() {
            let elem = &self.edges[self.index];
            self.index += 1;
            if elem.from == self.handle {
                return Some(elem.to);
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct Predecessors<'a> {
    handle: BlockHandle,
    edges: &'a Vec<CtrlFlowEdge>,
    index: usize,
}

impl<'a> Predecessors<'a> {
    fn new(handle: BlockHandle, edges: &'a Vec<CtrlFlowEdge>) -> Self {
        Self {
            handle,
            edges,
            index: 0,
        }
    }
}

impl<'a> Iterator for Predecessors<'a> {
    type Item = BlockHandle;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.edges.len() {
            let elem = &self.edges[self.index];
            self.index += 1;
            if elem.to == self.handle {
                return Some(elem.from);
            }
        }
        None
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
