use crate::GAVarNo;

#[derive(PartialEq, Eq, Copy, Clone)]
struct NodeNo(u32);
#[derive(PartialEq, Eq, Copy, Clone)]
struct EdgeNo(u32);

impl NodeNo {
    fn none() -> NodeNo {
        NodeNo(u32::MAX)
    }
}

impl EdgeNo {
    fn none() -> EdgeNo {
        EdgeNo(u32::MAX)
    }
}

struct Node {
    var: GAVarNo,
    next_edge: EdgeNo,
}

struct Edge {
    node: [NodeNo; 2],
    next_edge: [EdgeNo; 2],
}

impl Edge {
    fn edge_for_node(&self, node: NodeNo) -> EdgeNo {
        if self.node[0] == node {
            self.next_edge[0]
        } else {
            self.next_edge[1]
        }
    }
}

pub struct IFG {
    edges: Vec<Edge>,
    nodes: Vec<Node>,
}

impl IFG {
    pub fn insert_var(&mut self, var: GAVarNo) -> NodeNo {
        let index = NodeNo(self.nodes.len() as u32);
        self.nodes.push(Node {
            var,
            next_edge: EdgeNo::none(),
        });
        index
    }
}
