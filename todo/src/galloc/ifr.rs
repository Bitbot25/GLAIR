use crate::il::{reg::MachineReg, ILSized};

use super::liveness::LiveRange;
use std::fmt;

#[derive(PartialEq, Eq, Copy, Clone)]
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

impl fmt::Debug for NodeIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_some() {
            write!(f, "NodeIndex({})", self.0)
        } else {
            write!(f, "NodeIndex(None)")
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone)]
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

impl fmt::Debug for EdgeIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_some() {
            write!(f, "NodeIndex({})", self.0)
        } else {
            write!(f, "NodeIndex(None)")
        }
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

    dsatur_saturation: usize,
    color: Option<MachineReg>,
}

impl<T: Copy> Copy for Node<T> {}
impl<T: Clone> Clone for Node<T> {
    fn clone(&self) -> Self {
        Node {
            next_edge: self.next_edge,
            data: self.data.clone(),
            dsatur_saturation: 0,
            color: None,
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Node")
            .field("data", &self.data)
            .field("next_edge", &self.next_edge)
            .field("dsatur_saturation", &self.dsatur_saturation)
            .field("color", &self.color)
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
            dsatur_saturation: 0,
            color: None,
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
            let next_edge_choose = usize::from(current_edge.node[0] != base_node);
            let next_edge = current_edge.next_edges[next_edge_choose];

            edge = next_edge;
        }
        result
    }
}

pub fn dsatur_choose_uncolored_node<T>(ifr_graph: &mut InterferenceGraph<T>) -> Option<NodeIndex> {
    let mut max_node: Option<NodeIndex> = None;
    for (i, node) in ifr_graph.nodes.iter().enumerate() {
        if node.color.is_some() {
            continue;
        }
        match &mut max_node {
            Some(max_node_index) => {
                // TODO: Tiebreakers
                let max_node = ifr_graph.get_node(*max_node_index);
                if node.dsatur_saturation > max_node.dsatur_saturation {
                    *max_node_index = NodeIndex(i);
                }
            }
            None => max_node = Some(NodeIndex(i)),
        }
    }
    max_node
}

pub fn dsatur<T: ILSized>(ifr_graph: &mut InterferenceGraph<T>) {
    // FIXME: Support for more than AMD
    while let Some(node_index) = dsatur_choose_uncolored_node(ifr_graph) {
        // TODO: Optimise this
        let neighbor_colors: Vec<MachineReg> = ifr_graph
            .neighbors_vec(node_index)
            .into_iter()
            .filter_map(|neighbor_index| ifr_graph.get_node(neighbor_index).color)
            .collect();

        let node = ifr_graph.get_node(node_index);
        let color = MachineReg::amd_gpr_registers()
            .filter(|reg| {
                for neighbor_color in &neighbor_colors {
                    if neighbor_color.overlaps(reg) {
                        return false;
                    }
                }
                return true;
            })
            .filter(|reg| reg.il_size() == node.data().il_size())
            .next()
            .expect("Spilling is not implemented");
        ifr_graph.get_node_mut(node_index).color = Some(color);

        let saturation_affected_nodes = ifr_graph.neighbors_vec(node_index);
        for saturation_affected_node in saturation_affected_nodes {
            ifr_graph
                .get_node_mut(saturation_affected_node)
                .dsatur_saturation += 1;
        }
    }
}

pub fn construct(ranges: Vec<LiveRange>) -> InterferenceGraph<LiveRange> {
    let mut graph = InterferenceGraph::new();

    for range in ranges {
        graph.add_node(range);
    }

    let nodes_len = graph.nodes.len();
    for i in 0..nodes_len {
        for j in i + 1..nodes_len {
            // TODO: We have to access node_a in every iteration because of the borrow checker :(
            let node_a = &graph.nodes[i];
            let node_b = &graph.nodes[j];
            if !graph.has_edge(NodeIndex(i), NodeIndex(j)) && node_a.data().overlaps(node_b.data())
            {
                graph.add_edge(NodeIndex(i), NodeIndex(j));
            }
        }
    }

    /*for node in graph.nodes {
        let data = node.data();
    }*/

    graph
}
