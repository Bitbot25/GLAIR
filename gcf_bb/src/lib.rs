use std::fmt;
use std::marker::PhantomData;

use smallvec::SmallVec;

mod iter;
#[cfg(test)]
mod tests;
pub use iter::*;

pub struct BasicBlock<I> {
    instructions: SmallVec<[I; 4]>,
}

impl<I> BasicBlock<I> {
    pub fn new(instructions: SmallVec<[I; 4]>) -> Self {
        Self { instructions }
    }

    pub fn instructions(&self) -> &SmallVec<[I; 4]> {
        &self.instructions
    }
}

struct BasicBlockNode<I> {
    inner: BasicBlock<I>,
    next_edge: ControlFlowEdgeId,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct ControlFlowEdgeId(usize);

impl fmt::Debug for ControlFlowEdgeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self == &Self::_internal_none() {
            write!(f, "ControlFlowEdgeId(None)")
        } else {
            write!(f, "ControlFlowEdgeId({})", self.0)
        }
    }
}

impl ControlFlowEdgeId {
    #[inline]
    const fn _internal_none() -> Self {
        ControlFlowEdgeId(usize::max_value())
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct BasicBlockId(usize);

impl fmt::Debug for BasicBlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self == &Self::_internal_none() {
            write!(f, "BasicBlockId(None)")
        } else {
            write!(f, "BasicBlockId({})", self.0)
        }
    }
}

impl BasicBlockId {
    #[inline]
    const fn _internal_none() -> Self {
        BasicBlockId(usize::max_value())
    }
}

struct Edge {
    connection: [BasicBlockId; 2],
    next_edges: [ControlFlowEdgeId; 2],
}

pub trait Config {
    type EdgeRemoveMode;
}

pub struct ControlFlow<I> {
    edges: Vec<Edge>,
    blocks: Vec<BasicBlockNode<I>>,
}

impl<'this, I> ControlFlow<I>
where
    Self: 'this,
{
    #[inline]
    pub fn new() -> Self {
        ControlFlow {
            edges: Vec::new(),
            blocks: Vec::new(),
        }
    }

    #[inline]
    pub fn basic_block(&self, block_id: BasicBlockId) -> &BasicBlock<I> {
        &self.blocks[block_id.0].inner
    }

    #[inline]
    pub fn basic_block_mut(&mut self, block_id: BasicBlockId) -> &mut BasicBlock<I> {
        &mut self.blocks[block_id.0].inner
    }

    #[inline]
    pub fn predecessors(&self, block_id: BasicBlockId) -> Predecessors<I> {
        Predecessors {
            graph: self,
            edge: self.blocks[block_id.0].next_edge,

            #[cfg(debug_assertions)]
            original_block: block_id,
        }
    }

    #[inline]
    pub fn descendants(&self, block_id: BasicBlockId) -> Descendants<I> {
        eprintln!(
            "create descendants with {:?} as first edge",
            self.blocks[block_id.0].next_edge
        );
        Descendants {
            graph: self,
            original_block: block_id,
            edge: self.blocks[block_id.0].next_edge,
        }
    }

    #[inline]
    pub fn edges(&self, block_id: BasicBlockId) -> Edges<I> {
        Edges {
            graph: self,
            edge: self.blocks[block_id.0].next_edge,
            original_block: block_id,
        }
    }

    pub fn add_basic_block(&'this mut self, block: BasicBlock<I>) -> BasicBlockId {
        let index = BasicBlockId(self.blocks.len());
        self.blocks.push(BasicBlockNode {
            inner: block,
            next_edge: ControlFlowEdgeId::_internal_none(),
        });
        index
    }

    pub fn add_edge(&mut self, from: BasicBlockId, to: BasicBlockId) -> ControlFlowEdgeId {
        let from_node = &self.blocks[from.0];
        let to_node = &self.blocks[to.0];

        let edge_index = ControlFlowEdgeId(self.edges.len());
        let edge = Edge {
            connection: [from, to],
            next_edges: [from_node.next_edge, to_node.next_edge],
        };
        self.edges.push(edge);

        self.blocks[from.0].next_edge = edge_index;
        self.blocks[to.0].next_edge = edge_index;
        edge_index
    }
}

impl<'this, I> ControlFlow<I> {
    pub fn remove_block(&mut self, id: BasicBlockId) -> BasicBlock<I> {
        let removed_block = self.blocks.remove(id.0);

        // Remove references to removed_block
        self.edges.retain_mut(|e| {
            let from_id = e.connection[0];
            let to_id = e.connection[1];

            // TODO: Avoid setting both next_edges if we know we're gonna remove one of them
            if to_id == id || from_id == id {
                let from = &mut self.blocks[from_id.0];
                from.next_edge = e.next_edges[0];

                let to = &mut self.blocks[to_id.0];
                to.next_edge = e.next_edges[1];
                false
            } else {
                true
            }
        });
        removed_block.inner
    }

    pub fn remove_edge(&mut self, id: ControlFlowEdgeId) -> Option<(BasicBlockId, BasicBlockId)> {
        // TODO: Is there any way to remove some if-statements?
        // TODO: Remove redundant double memory accesses in the loops.
        // TODO: Convert this into two subroutines. One where there is only one node. That will probably speed this up, and make it easier to understand.

        if id.0 >= self.edges.len() {
            return None;
        }

        let old_last_edge_id = ControlFlowEdgeId(self.edges.len() - 1);
        let removed_edge = self.edges.swap_remove(id.0);
        // Remove references to edge

        // First from the directly connected nodes
        let from = &mut self.blocks[removed_edge.connection[0].0];
        if from.next_edge == id {
            from.next_edge = ControlFlowEdgeId::_internal_none();
        }

        let to = &mut self.blocks[removed_edge.connection[1].0];
        if to.next_edge == id {
            to.next_edge = ControlFlowEdgeId::_internal_none();
        }

        // Then from the other edges
        // Also perform the same step for updating the last edge index early
        for edge in &mut self.edges {
            if edge.next_edges[0] == id {
                edge.next_edges[0] = ControlFlowEdgeId::_internal_none();
            }
            if edge.next_edges[1] == id {
                edge.next_edges[1] = ControlFlowEdgeId::_internal_none();
            }

            // Perform the same step for the last node that was swapped
            if edge.next_edges[0] == old_last_edge_id {
                edge.next_edges[0] = id;
            }

            if edge.next_edges[1] == old_last_edge_id {
                edge.next_edges[1] = id;
            }
        }

        // If we've removed the only node, then this will fail. So check if it's empty.
        if !self.edges.is_empty() {
            let last_edge = &self.edges[id.0];
            // Update references to the moved edge

            // From the directly connected nodes
            let from = &mut self.blocks[last_edge.connection[0].0];
            if from.next_edge == old_last_edge_id {
                from.next_edge = id;
            }

            let to = &mut self.blocks[last_edge.connection[1].0];
            if to.next_edge == old_last_edge_id {
                to.next_edge = id;
            }
        }

        Some((removed_edge.connection[0], removed_edge.connection[1]))
    }
}
