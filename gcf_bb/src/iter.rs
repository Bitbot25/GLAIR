use crate::{BasicBlockId, ControlFlow, ControlFlowEdgeId};

pub struct Predecessors<'g, I> {
    pub(crate) graph: &'g ControlFlow<I>,
    pub(crate) edge: ControlFlowEdgeId,

    #[cfg(debug_assertions)]
    pub(crate) original_block: BasicBlockId,
}

impl<'g, I> Iterator for Predecessors<'g, I> {
    type Item = BasicBlockId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.edge == ControlFlowEdgeId::_internal_none() {
            return None;
        }
        dbg!(self.edge);
        let edge = &self.graph.edges[self.edge.0];
        let from = edge.connection[0];

        if from == self.original_block {
            None
        } else {
            self.edge = edge.next_edges[1];
            Some(from)
        }
    }
}

pub struct Descendants<'g, I> {
    pub(crate) graph: &'g ControlFlow<I>,
    pub(crate) original_block: BasicBlockId,
    pub(crate) edge: ControlFlowEdgeId,
}

impl<'g, I> Iterator for Descendants<'g, I> {
    type Item = BasicBlockId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.edge == ControlFlowEdgeId::_internal_none() {
            return None;
        }
        dbg!(self.edge);
        let edge = &self.graph.edges[self.edge.0];
        let to = edge.connection[1];

        if to == self.original_block {
            None
        } else {
            self.edge = edge.next_edges[0];
            Some(to)
        }
    }
}

pub struct Edges<'g, I> {
    pub(crate) graph: &'g ControlFlow<I>,
    pub(crate) original_block: BasicBlockId,
    pub(crate) edge: ControlFlowEdgeId,
}

impl<'g, I> Iterator for Edges<'g, I> {
    type Item = BasicBlockId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.edge == ControlFlowEdgeId::_internal_none() {
            return None;
        }
        let edge = &self.graph.edges[self.edge.0];
        let from = edge.connection[0];
        let to = edge.connection[1];

        if from == self.original_block {
            self.edge = self.graph.blocks[to.0].next_edge;
            Some(to)
        } else {
            self.edge = self.graph.blocks[from.0].next_edge;
            Some(from)
        }
    }
}
