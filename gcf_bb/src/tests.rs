use crate::{BasicBlock, ControlFlow};
use smallvec::smallvec;

#[test]
fn graph_retains_data() {
    let mut ctrl: ControlFlow<u32> = ControlFlow::new();
    let bb0 = ctrl.add_basic_block(BasicBlock::new(smallvec![1, 2, 3]));
    assert_eq!(ctrl.basic_block(bb0).instructions().as_slice(), [1, 2, 3]);
}

#[test]
fn graph_descendants() {
    let mut ctrl = ControlFlow::new();
    let bb0 = ctrl.add_basic_block(BasicBlock::new(smallvec![1, 2, 3]));
    let bb1 = ctrl.add_basic_block(BasicBlock::new(smallvec![4, 5, 6]));
    let bb0_bb1 = ctrl.add_edge(bb0, bb1);

    assert_eq!(
        ctrl.descendants(bb0).next(),
        Some(bb1),
        "BB1 is expected to come after BB0"
    );
    assert_eq!(
        ctrl.descendants(bb1).next(),
        None,
        "BB1 should have no descendants"
    );

    ctrl.remove_edge(bb0_bb1);
    assert_eq!(
        ctrl.descendants(bb0).next(),
        None,
        "BB0 should have no descendants after removing edge"
    );

    assert_eq!(
        ctrl.descendants(bb1).next(),
        None,
        "BB1 should still not have any descendants after removing edge"
    );
}

#[test]
fn graph_predecessors() {
    let mut ctrl = ControlFlow::new();
    let bb0 = ctrl.add_basic_block(BasicBlock::new(smallvec![1, 2, 3]));
    let bb1 = ctrl.add_basic_block(BasicBlock::new(smallvec![4, 5, 6]));
    let bb0_bb1 = ctrl.add_edge(bb0, bb1);

    assert_eq!(
        ctrl.predecessors(bb0).next(),
        None,
        "BB0 is expected to have no predecessors",
    );

    assert_eq!(
        ctrl.predecessors(bb1).next(),
        Some(bb0),
        "BB1 is expected to have BB0 as a predecessor",
    );

    ctrl.remove_edge(bb0_bb1);
    assert_eq!(
        ctrl.predecessors(bb0).next(),
        None,
        "BB0 should still not have any predecessors after removing the edge",
    );

    assert_eq!(
        ctrl.predecessors(bb1).next(),
        None,
        "BB1's edge to it's predecessors was removed and is not expected to exist"
    );
}
