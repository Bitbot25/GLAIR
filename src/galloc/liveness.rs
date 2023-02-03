use crate::il::{
    cfg::{BlockHandle, CtrlFlow, Location},
    Instruction, SSARegister,
};

pub fn find_deaths(var: &SSARegister, begin: BlockHandle, cfg: &CtrlFlow) -> Vec<Location> {
    let mut list = Vec::new();
    internal_find_deaths(var, begin, 0, &mut list, cfg);
    list
}

// TODO: Create a non-recursive version
fn internal_find_deaths(
    var: &SSARegister,
    current_block: BlockHandle,
    mut branch_id: usize,
    deaths: &mut Vec<Location>,
    cfg: &CtrlFlow,
) {
    fn insert_use(list: &mut Vec<Location>, branch_id: usize, location: Location) {
        if branch_id >= list.len() {
            assert_eq!(
                branch_id,
                list.len(),
                "Locations can only be added one at a time."
            );
            list.push(location);
        } else {
            list[branch_id] = location;
        }
    }

    for (ins_offset, ins) in cfg
        .realise_handle(current_block)
        .instructions()
        .iter()
        .enumerate()
    {
        if ins.loaded_variables().contains(&var) {
            insert_use(deaths, branch_id, Location::new(current_block, ins_offset));
        }
    }

    for descendant in cfg.descendants(current_block) {
        internal_find_deaths(var, descendant, branch_id, deaths, cfg);
        branch_id += 1;
    }
}

pub trait LivenessAccumulator {
    fn is_marked(&self, reg: &SSARegister, loc: Location) -> bool;
    fn mark(&mut self, reg: &SSARegister, loc: Location);
}

pub fn mark_live_in_range<A>(
    var: &SSARegister,
    begin: Location,
    end: Location,
    accumulator: &mut A,
    graph: &CtrlFlow,
) where
    A: LivenessAccumulator,
{
    internal_mark_live_in_range(var, begin, end, accumulator, graph);
}

fn internal_mark_live_in_range<A>(
    var: &SSARegister,
    begin: Location,
    end: Location,
    accumulator: &mut A,
    graph: &CtrlFlow,
) where
    A: LivenessAccumulator,
{
    // TODO: Optimise this
    for (offset, ins) in graph
        .realise_handle(end.block_handle())
        .instructions()
        .iter()
        .enumerate()
        .rev()
    {
        let cur_location = Location::new(end.block_handle(), offset);
        // This is a branch that we've already marked
        if accumulator.is_marked(var, cur_location) {
            return;
        }

        let is_before_end = cur_location == end || cur_location.is_before(graph, &end);
        let is_after_begin = cur_location == begin || cur_location.is_after(graph, &begin);
        if is_before_end && is_after_begin {
            accumulator.mark(var, cur_location);
        }
    }

    for predecessor in graph.predecessors(end.block_handle()) {
        internal_mark_live_in_range(
            var,
            begin,
            Location::new(
                predecessor,
                predecessor.realise(graph).instructions().len() - 1,
            ),
            accumulator,
            graph,
        );
    }
}
