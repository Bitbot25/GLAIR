use std::collections::HashMap;

use crate::il::{
    cfg::{BlockHandle, CtrlFlow, Location},
    SSARegister,
};

#[derive(Debug, Copy, Clone)]
pub struct LiveSegment {
    block: BlockHandle,
    begin: usize,
    end: usize,
}

impl LiveSegment {
    pub fn begin_offset(&self) -> usize {
        self.begin
    }

    pub fn end_offset(&self) -> usize {
        self.end
    }

    pub fn begin(&self) -> Location {
        Location::new(self.block, self.begin)
    }

    pub fn end(&self) -> Location {
        Location::new(self.block, self.end)
    }

    pub fn is_adjacent_with(&self, other: &LiveSegment) -> bool {
        self.block == other.block && {
            let end_is_adjacent = self.end + 1 == other.begin;
            let begin_is_adjacent = other.end + 1 == self.begin;
            end_is_adjacent || begin_is_adjacent
        }
    }

    pub fn expand_to(&mut self, other: &LiveSegment) {
        self.begin = self.begin.min(other.begin);
        self.end = self.end.max(other.end);
    }

    #[inline]
    pub fn contains(&self, location: Location) -> bool {
        self.block == location.block_handle() && {
            location.offset() >= self.begin && location.offset() < self.end
        }
    }

    #[inline]
    pub fn overlaps(&self, other: &LiveSegment) -> bool {
        self.block == other.block && { self.begin <= other.end && other.begin <= self.end }
    }
}

#[derive(Default)]
pub struct LiveRangesBuilder<'a> {
    locations: HashMap<&'a SSARegister, Vec<Location>>,
}

impl<'a> LiveRangesBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mark(&mut self, reg: &'a SSARegister, location: Location) {
        match self.locations.get_mut(reg) {
            Some(locations) => {
                locations.push(location);
            }
            None => {
                self.locations.insert(reg, vec![location]);
            }
        }
    }

    pub fn build(self) -> Vec<LiveRange> {
        let mut out = Vec::new();
        for (reg, locations) in self.locations {
            let merged = merge_to_live_segments(locations);
            out.push(LiveRange {
                segments: merged,
                reg: *reg,
            });
        }
        out
    }
}

#[derive(Debug)]
pub struct LiveRange {
    segments: Vec<LiveSegment>,
    reg: SSARegister,
}

impl LiveRange {
    pub fn overlaps(&self, other: &LiveRange) -> bool {
        // TODO: Optimise this
        for segment in &self.segments {
            if let Some(other) = other.segment_in(segment.block) {
                if segment.overlaps(other) {
                    return true;
                }
            }
        }
        false
    }

    #[inline]
    pub fn contains(&self, location: Location) -> bool {
        self.segments
            .iter()
            .any(|segment| segment.contains(location))
    }

    pub fn segment_in(&self, block: BlockHandle) -> Option<&LiveSegment> {
        self.segments.iter().find(|&segment| segment.block == block)
    }

    pub fn segments(&self) -> &Vec<LiveSegment> {
        &self.segments
    }

    pub fn reg(&self) -> &SSARegister {
        &self.reg
    }
}

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

/// Merge adjacent locations into ranges of variable length. This is easier on the register allocator than [`strict_to_ranges`], but might yield worse results.
fn merge_to_live_segments(locations: Vec<Location>) -> Vec<LiveSegment> {
    // TODO: optimise this
    let mut segments = Vec::new();

    // Initialise the ranges
    for loc in locations {
        segments.push(Some(LiveSegment {
            block: loc.block_handle(),
            begin: loc.offset(),
            end: loc.offset() + 1,
        }));
    }

    // Now merge
    let segments_len = segments.len();
    // SAFETY: Taking the pointer here and using it later is safe because we're not using any function that's reallocating the array
    let segments_ptr = segments.as_ptr();
    let mut found_adjacent = true;
    while found_adjacent {
        found_adjacent = false;
        // TODO: This is really bad...
        let mut i = 0;
        while i < segments_len {
            let mut j = i + 1;
            let seg_ref = &mut segments[i];
            i += 1;
            let seg = match seg_ref {
                Some(seg) => seg,
                None => continue,
            };
            while j < segments_len {
                // SAFETY: seg and other_ref will never point to the same element
                let other_ref = unsafe { &mut *(segments_ptr.add(j) as *mut _) };
                j += 1;
                let other = match other_ref {
                    Some(other) => other,
                    None => continue,
                };
                if seg.is_adjacent_with(other) || seg.overlaps(other) {
                    seg.expand_to(other);
                    *other_ref = None;
                    found_adjacent = true;
                }
            }
        }
    }

    segments.into_iter().flatten().collect()
}

pub fn mark_live_in_range<'a>(
    var: &'a SSARegister,
    begin: Location,
    end: Location,
    ranges: &mut LiveRangesBuilder<'a>,
    graph: &CtrlFlow,
) {
    internal_mark_live_in_range(var, begin, end, ranges, graph);
}

fn internal_mark_live_in_range<'a>(
    var: &'a SSARegister,
    begin: Location,
    end: Location,
    ranges: &mut LiveRangesBuilder<'a>,
    graph: &CtrlFlow,
) {
    // TODO: Optimise this
    for (offset, _ins) in graph
        .realise_handle(end.block_handle())
        .instructions()
        .iter()
        .enumerate()
        .rev()
    {
        let cur_location = Location::new(end.block_handle(), offset);

        // This is a branch that we've already marked
        //if accumulator.is_marked(var, cur_location) {
        //    return;
        //}

        let is_before_end = cur_location == end || cur_location.is_before(graph, &end);
        let is_after_begin = cur_location == begin || cur_location.is_after(graph, &begin);
        if is_before_end && is_after_begin {
            ranges.mark(var, cur_location);
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
            ranges,
            graph,
        );
    }
}
