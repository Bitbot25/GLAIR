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

struct Edge {
    start: NodeIndex,
    end: NodeIndex,

    /// Edge going from this edge
    next_edge: EdgeIndex,
}

pub struct Node<T> {
    data: T,

    /// Edge going from this node
    next_edge: EdgeIndex,
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

impl<T> InterferenceGraph<T> {
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

    /// Get the data for the node with [`index`] as its index
    #[inline]
    pub fn get_node(&self, index: NodeIndex) -> &Node<T> {
        &self.nodes[index.0]
    }

    #[inline]
    fn get_edge(&self, index: EdgeIndex) -> &Edge {
        &self.edges[index.0]
    }

    #[inline]
    pub fn neighbors_vec(&self, base_index: NodeIndex) -> Vec<NodeIndex> {
        let base_edge = self.get_node(base_index).next_edge;

        let mut prev_edge = base_edge;
        let mut result = Vec::new();
        while prev_edge.is_some() {
            let edge_index = self.get_edge(prev_edge).next_edge;
            let edge = self.get_edge(edge_index);
            let neighbor = if edge.end == base_index {
                edge.start
            } else {
                edge.end
            };
            result.push(neighbor);
            prev_edge = edge_index;
        }
        result
    }
}
