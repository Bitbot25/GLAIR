use std::{collections::HashMap, fmt};

use crate::il::{cfg::Location, SSARegister};

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
    pub fn has_edge(&mut self, a: NodeIndex, b: NodeIndex) -> bool {
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

pub struct InterferenceData {
    live: Vec<Location>,
}

impl InterferenceData {
    pub fn new(live: Vec<Location>) -> Self {
        Self { live }
    }
}

pub struct InterferenceAccum {
    map: HashMap<SSARegister, Vec<Location>>,
}

impl InterferenceAccum {
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

    pub fn mark_live_at(&mut self, reg: &SSARegister, loc: Location) {
        match self.map.get_mut(reg) {
            Some(list) => list.push(loc),
            None => {
                self.map.insert(*reg, vec![loc]);
            }
        }
    }
}

impl LivenessAccumulator for InterferenceAccum {
    fn mark(&mut self, reg: &SSARegister, loc: Location) {
        self.mark_live_at(reg, loc)
    }

    fn is_marked(&self, reg: &SSARegister, loc: Location) -> bool {
        self.is_live_at(reg, loc)
    }
}

impl IntoIterator for InterferenceAccum {
    type Item = (SSARegister, Vec<Location>);
    type IntoIter = impl Iterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

pub fn construct_ssa(
    program_variables: impl Iterator<Item = (SSARegister, InterferenceData)>,
) -> InterferenceGraph<SSARegister> {
    let mut graph = InterferenceGraph::new();
    let mut ifr_set: HashMap<Location, Vec<&SSARegister>> = HashMap::new();
    let mut handle_map: HashMap<&SSARegister, NodeIndex> = HashMap::new();

    // TODO: Maybe a solution without collecting the vector?
    let program_variables: Vec<(SSARegister, InterferenceData)> = program_variables.collect();

    for (var, ifr_data) in &program_variables {
        let handle = graph.add_node(*var);
        handle_map.insert(var, handle);
        for loc in &ifr_data.live {
            match ifr_set.get_mut(loc) {
                Some(list) => {
                    list.push(&var);
                }
                None => {
                    ifr_set.insert(*loc, vec![&var]);
                }
            }
        }
    }
    for cluster in ifr_set.values() {
        for (it, var) in cluster.iter().enumerate() {
            let var = handle_map[var];
            for connection in &cluster[it..] {
                let connection = handle_map[connection];
                if connection == var {
                    continue;
                }

                if !graph.has_edge(var, connection) {
                    graph.add_edge(var, connection);
                }
            }
        }
    }
    graph
}
