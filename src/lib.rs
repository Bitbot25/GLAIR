#![feature(type_alias_impl_trait)]

pub mod galloc;
pub mod il;
#[cfg(all(target_os = "linux", target_pointer_width = "64"))]
pub mod linux64;

#[cfg(test)]
mod tests {
    use crate::galloc::ifr::InterferenceGraph;

    #[test]
    fn ifr_graph() {
        let mut ifr_graph = InterferenceGraph::new();
        let n0 = ifr_graph.add_node(0);
        let n1 = ifr_graph.add_node(1);
        let n2 = ifr_graph.add_node(2);

        ifr_graph.add_edge(n0, n1);
        ifr_graph.add_edge(n0, n2);

        assert_eq!(ifr_graph.neighbors_vec(n0), vec![n2, n1]);
    }
}
