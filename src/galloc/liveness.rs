use crate::il::{
    cfg::{BlockHandle, CtrlFlow, LocalRange, Location},
    SSARegister,
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

pub trait LivenessAccumulator<'a> {
    fn is_marked(&self, reg: &SSARegister, loc: Location) -> bool;
    fn mark(&mut self, reg: &'a SSARegister, loc: Location);
}

struct ZipAny<A, B> {
    a: A,
    b: B,
}

impl<A, B> ZipAny<A, B> {
    fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

enum ZipAnyPair<T> {
    First(T),
    Second(T),
    Both(T, T),
}

impl<T, A, B> Iterator for ZipAny<A, B>
where
    A: Iterator<Item = T>,
    B: Iterator<Item = T>,
{
    type Item = ZipAnyPair<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let a_res = self.a.next();
        let b_res = self.b.next();
        match a_res {
            Some(a) => match b_res {
                Some(b) => Some(ZipAnyPair::Both(a, b)),
                None => Some(ZipAnyPair::First(a)),
            },
            None => match b_res {
                Some(b) => Some(ZipAnyPair::Second(b)),
                None => None,
            },
        }
    }
}

/// Simply convert the locations to ranges of length 1. This is harder on the register allocator / ifr graph, but might give more intelligent results.
pub fn strict_to_ranges(locations: Vec<Location>, cfg: &CtrlFlow) -> Vec<LocalRange> {
    let mut ranges = Vec::new();
    for loc in locations {
        ranges.push(LocalRange::point(loc));
    }
    ranges
}

// TODO: Make additional splits where variables die so that the register allocator has more opportunities?
/// Merge adjacent locations into ranges of variable length. This is easier on the register allocator than [`strict_to_ranges`], but might yield worse results.
pub fn merge_to_ranges(locations: Vec<Location>, cfg: &CtrlFlow) -> Vec<LocalRange> {
    // TODO: optimise this
    let mut ranges: Vec<LocalRange> = Vec::new();
    'a: for loc in locations {
        for range in &mut ranges {
            if loc.block_handle() != range.block() {
                continue;
            }
            if range.loc_intersects(&loc) {
                // This location is already featured in a range, skip this one.
                continue 'a;
            }

            if loc.offset() == range.to() + 1 {
                *range = LocalRange::new(range.from(), loc.offset(), range.block());
                continue 'a;
            } else if range.from() != 0 && loc.offset() == range.from() - 1 {
                *range = LocalRange::new(loc.offset(), range.to(), range.block());
                continue 'a;
            }
        }
        ranges.push(LocalRange::point(loc));
    }
    ranges
}

pub fn mark_live_in_range<'a, A>(
    var: &'a SSARegister,
    begin: Location,
    end: Location,
    accumulator: &mut A,
    graph: &CtrlFlow,
) where
    A: LivenessAccumulator<'a>,
{
    internal_mark_live_in_range(var, begin, end, accumulator, graph);
}

fn internal_mark_live_in_range<'a, A>(
    var: &'a SSARegister,
    begin: Location,
    end: Location,
    accumulator: &mut A,
    graph: &CtrlFlow,
) where
    A: LivenessAccumulator<'a>,
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
                graph.realise_handle(predecessor).instructions().len() - 1,
            ),
            accumulator,
            graph,
        );
    }
}
