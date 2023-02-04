use std::{collections::HashMap, fmt};

use crate::il::{
    cfg::{CtrlFlow, LocalRange, Location},
    SSARegister,
};

use super::liveness::LivenessAccumulator;

// FIXME: Manual debug impls for NodeIndex and EdgeIndex because it looks wierd when it's none

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct NodeIndex(usize);

impl NodeIndex {
    #[inline]
    pub fn none() -> Self {
        NodeIndex(usize::max_value())
    }

    #[inline]
    pub fn is_none(&self) -> bool {
        *self == Self::none()
    }

    #[inline]
    pub fn is_some(&self) -> bool {
        !self.is_none()
    }
}
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct EdgeIndex(usize);

impl EdgeIndex {
    #[inline]
    pub fn none() -> Self {
        EdgeIndex(usize::max_value())
    }

    #[inline]
    pub fn is_none(&self) -> bool {
        *self == Self::none()
    }

    #[inline]
    pub fn is_some(&self) -> bool {
        !self.is_none()
    }
}

#[derive(Debug)]
struct Edge {
    /// The two nodes this edge connects
    node: [NodeIndex; 2],

    /// Edge going from this edge
    /// There are 2 because one is going from node[0] and the other node[1]
    next_edges: [EdgeIndex; 2],
}

pub struct Node<T> {
    data: T,

    /// Edge going from this node
    next_edge: EdgeIndex,
}

impl<T: Copy> Copy for Node<T> {}
impl<T: Clone> Clone for Node<T> {
    fn clone(&self) -> Self {
        Node {
            next_edge: self.next_edge,
            data: self.data.clone(),
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Node")
            .field("data", &self.data)
            .field("next_edge", &self.next_edge)
            .finish()
    }
}

impl<T> Node<T> {
    #[inline]
    pub fn data(&self) -> &T {
        &self.data
    }
}

pub struct InterferenceGraph<T> {
    nodes: Vec<Node<T>>,
    edges: Vec<Edge>,
}

impl<T: fmt::Debug> fmt::Debug for InterferenceGraph<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InterferenceGraph")
            .field("nodes", &self.nodes)
            .field("edges", &self.edges)
            .finish()
    }
}

impl<T> InterferenceGraph<T> {
    #[inline]
    pub fn new() -> Self {
        InterferenceGraph {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Add a node to the interference graph
    #[inline]
    pub fn add_node(&mut self, data: T) -> NodeIndex {
        let index = self.nodes.len();
        let node = Node {
            data,
            next_edge: EdgeIndex::none(),
        };
        self.nodes.push(node);
        NodeIndex(index)
    }

    #[inline]
    pub fn add_edge(&mut self, a: NodeIndex, b: NodeIndex) -> EdgeIndex {
        let index = self.edges.len();
        let edge_idx = EdgeIndex(index);
        let mut edge = Edge {
            node: [a, b],
            next_edges: [EdgeIndex::none(), EdgeIndex::none()],
        };

        let a_src = self.get_node_mut(a);
        edge.next_edges[0] = a_src.next_edge;
        a_src.next_edge = edge_idx;
        let b_src = self.get_node_mut(b);
        edge.next_edges[1] = b_src.next_edge;
        b_src.next_edge = edge_idx;

        self.edges.push(edge);
        edge_idx
    }

    #[inline]
    pub fn has_edge(&self, a: NodeIndex, b: NodeIndex) -> bool {
        // TODO: optimise this
        self.neighbors_vec(a).contains(&b)
    }

    /// Get the data for the node with [`index`] as its index
    #[inline]
    pub fn get_node(&self, index: NodeIndex) -> &Node<T> {
        &self.nodes[index.0]
    }

    /// Get the data for the node with [`index`] as its index
    #[inline]
    pub fn get_node_mut(&mut self, index: NodeIndex) -> &mut Node<T> {
        &mut self.nodes[index.0]
    }

    #[inline]
    fn get_edge(&self, index: EdgeIndex) -> &Edge {
        &self.edges[index.0]
    }

    #[inline]
    pub fn neighbors_vec(&self, base_node: NodeIndex) -> Vec<NodeIndex> {
        let base_edge = self.get_node(base_node).next_edge;

        let mut edge = base_edge;
        let mut result = Vec::new();
        while edge.is_some() {
            let current_edge = self.get_edge(edge);
            if current_edge.node[0] == base_node {
                result.push(current_edge.node[1]);
            } else {
                result.push(current_edge.node[0]);
            }
            let next_edge_choose = if current_edge.node[0] == base_node {
                0
            } else {
                1
            };
            let next_edge = current_edge.next_edges[next_edge_choose];

            edge = next_edge;
        }
        result
    }
}

#[derive(Debug)]
pub struct InterferenceData<'a> {
    ranges: Vec<LocalRange>,
    variable: &'a SSARegister,
}

impl<'a> InterferenceData<'a> {
    pub fn new(ranges: Vec<LocalRange>, variable: &'a SSARegister) -> Self {
        Self { ranges, variable }
    }
}

pub struct InterferenceAccum<'a> {
    map: HashMap<&'a SSARegister, Vec<Location>>,
}

impl<'a> InterferenceAccum<'a> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn is_live_at(&self, reg: &SSARegister, loc: Location) -> bool {
        match self.map.get(reg) {
            Some(list) => list.contains(&loc),
            None => false,
        }
    }

    pub fn mark_live_at(&mut self, reg: &'a SSARegister, loc: Location) {
        match self.map.get_mut(reg) {
            Some(list) => list.push(loc),
            None => {
                self.map.insert(reg, vec![loc]);
            }
        }
    }
}

impl<'a> LivenessAccumulator<'a> for InterferenceAccum<'a> {
    fn mark(&mut self, reg: &'a SSARegister, loc: Location) {
        self.mark_live_at(reg, loc)
    }

    fn is_marked(&self, reg: &SSARegister, loc: Location) -> bool {
        self.is_live_at(reg, loc)
    }
}

impl<'a> IntoIterator for InterferenceAccum<'a> {
    type Item = (&'a SSARegister, Vec<Location>);
    type IntoIter = impl Iterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct InterferenceRange<'a> {
    range: LocalRange,
    variable: &'a SSARegister,
}

impl<'a> InterferenceRange<'a> {
    fn new(range: LocalRange, variable: &'a SSARegister) -> Self {
        Self { range, variable }
    }

    fn intersects(&self, other: &InterferenceRange) -> bool {
        self.range.intersects(&other.range)
    }
}

pub fn construct<'var>(
    variable_interference: Vec<InterferenceData<'var>>,
    cfg: &CtrlFlow,
) -> InterferenceGraph<InterferenceRange<'var>> {
    let mut graph: InterferenceGraph<InterferenceRange> = InterferenceGraph::new();

    for InterferenceData { ranges, variable } in variable_interference {
        for live_range in ranges {
            graph.add_node(InterferenceRange::new(live_range, variable));
        }
    }

    // TODO: Optimise this so that we don't need to clone the nodes
    for (it, node) in graph.nodes.clone().into_iter().enumerate() {
        let node_ifr = node.data();
        let node_index = NodeIndex(it);
        for (o_idx, other) in graph.nodes.clone()[it + 1..].iter().enumerate() {
            let o_idx = o_idx + it + 1;
            let other_ifr = other.data();
            let other_index = NodeIndex(o_idx);
            if other_ifr.intersects(node_ifr) {
                if graph.has_edge(node_index, other_index) {
                    eprintln!("optimise: this node already has an edge");
                } else {
                    graph.add_edge(node_index, other_index);
                }
            }
        }
    }

    graph
}
