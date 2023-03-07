use gcf_bb as bb;
use std::fmt;
use smallvec::{smallvec, SmallVec};
use std::collections::{HashMap, HashSet, VecDeque};

use super::*;

#[derive(Debug)]
pub struct LocalLiveRange {
    from: LiveLocation,
    to: LiveLocation,
}

#[derive(Debug)]
pub struct LiveLocation {
    mode: LiveLocationMode,
    index: InstructionId,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LiveLocationMode {
    Pre,
    Post,
}

#[derive(Debug)]
pub struct LiveRange {
    ranges: Vec<LocalLiveRange>,
}

#[derive(Debug)]
pub struct Allocator {
    blocks_postorder: Vec<bb::BasicBlockId>,
    block_to_allocbb: HashMap<bb::BasicBlockId, AllocBB>,
    reg_to_liverange: HashMap<VrReg, LiveRange>,
    cfg: bb::ControlFlow<Instruction>,
}

#[derive(Debug)]
struct AllocBB {
    bb: bb::BasicBlockId,
    live_in: LiveSet,
    live_out: LiveSet,
    registers: HashSet<VrReg>,
    defs_reverse_order: Vec<LiveLocation>,
}

#[derive(Clone)]
struct LiveSet {
    vars: HashSet<VrReg>,
}

impl fmt::Debug for LiveSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.vars, f)
    }
}

impl LiveSet {
    fn new() -> Self {
        LiveSet {
            vars: HashSet::new(),
        }
    }

    fn union_add(&mut self, other: &LiveSet) {
        for v in &other.vars {
            self.vars.insert(*v);
        }
    }

    fn set(&mut self, reg: VrReg, b: bool) {
        if b {
            self.vars.insert(reg);
        } else {
            self.vars.remove(&reg);
        }
    }

    fn contains(&self, reg: VrReg) -> bool {
        self.vars.contains(&reg)
    }

    fn iter(&self) -> impl Iterator<Item = VrReg> + '_ {
        self.vars.iter().map(|x| *x)
    }
}

fn compute_postorder<I>(
    entry: bb::BasicBlockId,
    cfg: &bb::ControlFlow<I>,
) -> Vec<bb::BasicBlockId> {
    // Yanked straight from cranelift
    // TODO: Only compute successors and such once. Store it in a ControlFlowInfo.
    let num_blocks = cfg.basic_blocks().len();
    let mut ret = vec![];

    // State: visited-block map, and explicit DFS stack.
    let mut visited = vec![];
    visited.resize(num_blocks, false);

    struct State {
        block: bb::BasicBlockId,
        succs: SmallVec<[bb::BasicBlockId; 4]>,
        next_succ: usize,
    }
    let mut stack: SmallVec<[State; 64]> = smallvec![];

    visited[entry.as_usize()] = true;
    stack.push(State {
        block: entry,
        succs: cfg.successors(entry).collect(),
        next_succ: 0,
    });

    while let Some(ref mut state) = stack.last_mut() {
        // Perform one action: push to new succ, skip an already-visited succ, or pop.
        if state.next_succ < state.succs.len() {
            let succ = state.succs[state.next_succ];
            state.next_succ += 1;
            if !visited[succ.as_usize()] {
                visited[succ.as_usize()] = true;
                stack.push(State {
                    block: succ,
                    succs: cfg.successors(succ).collect(),
                    next_succ: 0,
                });
            }
        } else {
            ret.push(state.block);
            stack.pop();
        }
    }

    ret
}

impl Allocator {
    pub fn new(cfg: bb::ControlFlow<Instruction>) -> Self {
        let mut block_to_allocbb = HashMap::new();
        // Initialise allocbbs
        for block in cfg.basic_blocks() {
            block_to_allocbb.insert(
                block,
                AllocBB {
                    bb: block,
                    live_in: LiveSet::new(),
                    live_out: LiveSet::new(),
                    registers: HashSet::new(),
                    defs_reverse_order: Vec::new(),
                },
            );
        }

        let me = Allocator {
            block_to_allocbb,
            blocks_postorder: compute_postorder(cfg.entry(), &cfg),
            reg_to_liverange: HashMap::new(),
            cfg,
        };
        me
    }

    pub fn block_inout(&mut self) {
        // Initialise queue with post order (last blocks)
        let mut queue = VecDeque::with_capacity(self.blocks_postorder.len());
        let mut visited = HashSet::new();
        for b in &self.blocks_postorder {
            queue.push_back(*b);
        }

        while !queue.is_empty() {
            let block_id = queue.pop_front().unwrap();
            let block = self.cfg.basic_block(block_id);

            if visited.contains(&block_id) {
                continue;
            }
            visited.insert(block_id);

            // TODO: Optimise this :(
            let succs: Vec<_> = self
                .cfg
                .successors(block_id)
                .map(|id| self.block_to_allocbb[&id].live_in.clone())
                .collect();
            let allocbb = self.block_to_allocbb.get_mut(&block_id).unwrap();
            for succ_in in succs {
                allocbb.live_out.union_add(&succ_in);
            }

            let allocbb = &mut self.block_to_allocbb.get_mut(&block_id).unwrap();
            let live_in = &mut allocbb.live_in;
            let registers = &mut allocbb.registers;
            let block_defs_reverse = &mut allocbb.defs_reverse_order;

            for ins in block.instructions().iter().rev() {
                for mode in &[LiveLocationMode::Post, LiveLocationMode::Pre] {
                    for operand in &ins.operands {
                        if operand.pos_mode == *mode {
                            match operand.rw_mode {
                                RWMode::Def => {
                                    live_in.set(operand.reg, false);
                                    block_defs_reverse.push(LiveLocation {
                                        mode: *mode,
                                        index: ins.id(),
                                    });
                                }
                                RWMode::Use => live_in.set(operand.reg, true),
                            }
                        }
                    }
                }
            }
            for reg in allocbb.live_out.iter().chain(allocbb.live_in.iter()) {
                registers.insert(reg);
            }

            for pred in self.cfg.predecessors(block_id) {
                queue.push_back(pred);
            }
        }
    }

    fn insert_local_liverange(&mut self, vreg: VrReg, local: LocalLiveRange) {
        if let Some(liverange) = self.reg_to_liverange.get_mut(&vreg) {
            liverange.ranges.push(local);
        } else {
            self.reg_to_liverange.insert(
                vreg,
                LiveRange {
                    ranges: vec![local],
                },
            );
        }
    }

    /// CAUTION! ONLY WORKS WITH SSA
    pub fn block_liverange(&mut self) {
        for (block, allocbb) in &self.block_to_allocbb {
            let real_block = self.cfg.basic_block(*block);
            let insns = real_block.instructions();

            /*let mut live = allocbb.live_out.clone();
            // Assume that registers are live for the whole block
            for reg in live.iter() {
                let local_range = LocalLiveRange {
                    from: LiveLocation {
                        index: insns[0].id(),
                        mode: LiveLocationMode::Pre,
                    },
                    to: LiveLocation {
                        index: insns.last().unwrap().id(),
                        mode: LiveLocationMode::Post,
                    },
                };
                // TODO: Had to manually inline the insert_local_liverange function for borrow checker to be happy... Fix this later...
                if let Some(liverange) = self.reg_to_liverange.get_mut(&reg) {
                    liverange.ranges.push(local_range);
                } else {
                    self.reg_to_liverange.insert(
                        reg,
                        LiveRange {
                            ranges: vec![local_range],
                        },
                    );
                }
            }*/

            for reg in &allocbb.registers {
                let local_range = LocalLiveRange {
                    from: LiveLocation {
                        index: insns[0].id(),
                        mode: LiveLocationMode::Pre,
                    },
                    to: LiveLocation {
                        index: insns.last().unwrap().id(),
                        mode: LiveLocationMode::Post,
                    },
                };
                // TODO: Had to manually inline the insert_local_liverange function for borrow checker to be happy... Fix this later...
                let local_range_index = if let Some(liverange) = self.reg_to_liverange.get_mut(&reg) {
                    let index = liverange.ranges.len();
                    liverange.ranges.push(local_range);
                    index
                } else {
                    self.reg_to_liverange.insert(
                        *reg,
                        LiveRange {
                            ranges: vec![local_range],
                        },
                    );
                    0
                };


                // Set beginning of lifetime
                let entry = &mut self.reg_to_liverange.get_mut(&reg).unwrap().ranges[local_range_index];
                let set_begin = !allocbb.live_in.contains(*reg);
                let set_end   = !allocbb.live_out.contains(*reg);

                for ins in insns.iter().rev() {
                    if set_end {
                        for mode in &[LiveLocationMode::Pre, LiveLocationMode::Post] {
                            for operand in &ins.operands {
                                if operand.pos_mode != *mode || operand.reg != *reg || operand.rw_mode != RWMode::Use {
                                    continue;
                                }

                                entry.to = LiveLocation {
                                    mode: *mode,
                                    index: ins.id(),
                                };
                            }
                        }
                    }

                    if set_begin {
                        for mode in &[LiveLocationMode::Post, LiveLocationMode::Pre] {
                            for operand in &ins.operands {
                                if operand.pos_mode != *mode || operand.reg != *reg || operand.rw_mode != RWMode::Def {
                                    continue;
                                }

                                entry.from = LiveLocation {
                                    mode: *mode,
                                    index: ins.id(),
                                };
                            }
                        }
                    }
                }
            }

        }
    }
}
