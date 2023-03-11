use gcf_bb as bb;
use smallvec::{SmallVec, smallvec};
use std::collections::{HashMap, HashSet, VecDeque};

use live::LiveRange;

mod live;

#[derive(Hash, PartialEq, Eq, Debug, Copy, Clone)]
pub struct InstructionId(pub usize);

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub struct VrReg(pub usize);
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub struct HwReg(pub usize);

const X86_64_REGCNT: usize = 4;

impl HwReg {
    pub fn null() -> Self {
        HwReg(usize::MAX)
    }

    pub fn is_null(&self) -> bool {
        *self == Self::null()
    }

    pub fn iter() -> impl Iterator<Item = HwReg> {
        (0..X86_64_REGCNT).map(|n| HwReg(n))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct OperandId(pub usize);

#[derive(Debug, Copy, Clone)]
pub enum Fixation {
    Operand(OperandId),
    HwReg(HwReg),
    None,
}

#[derive(Debug, Copy, Clone)]
pub struct Operand {
    pub fixation: Fixation,
    pub id: OperandId,
    pub reg: VrReg,
    pub rw_mode: RWMode,
    pub pos_mode: live::LiveLocationMode,
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum RWMode {
    Use,
    Def,
}

#[derive(Debug)]
pub struct Instruction {
    id: InstructionId,
    operands: Vec<Operand>,
}

impl Instruction {
    pub fn new(id: InstructionId, operands: Vec<Operand>) -> Self {
        Instruction { id, operands }
    }

    pub fn id(&self) -> InstructionId {
        self.id
    }

    pub fn operands(&self) -> &Vec<Operand> {
        &self.operands
    }
}

#[derive(Debug)]
enum AllocationSpot {
    Stack(),
    Reg(HwReg),
}

#[derive(Debug)]
pub struct Allocation {
    liverange: LiveRange,
    loc: AllocationSpot,
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
    live_in: live::LiveSet,
    live_out: live::LiveSet,
    registers: HashSet<VrReg>,
    defs_reverse_order: Vec<live::LiveLocation>,
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
                    live_in: live::LiveSet::new(),
                    live_out: live::LiveSet::new(),
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
                for mode in &[live::LiveLocationMode::Post, live::LiveLocationMode::Pre] {
                    for operand in &ins.operands {
                        if operand.pos_mode == *mode {
                            match operand.rw_mode {
                                RWMode::Def => {
                                    live_in.set(operand.reg, false);
                                    block_defs_reverse.push(live::LiveLocation {
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

    pub fn allocate(self)-> Vec<Allocation> {
        let mut stack = self.reg_to_liverange.into_values().collect::<Vec<_>>();
        let mut hw_registers = HwReg::iter();
        let mut allocs = Vec::with_capacity(stack.len());
        while !stack.is_empty() {
            let liverange = stack.pop().unwrap();
            let loc = hw_registers
                .next()
                .map(AllocationSpot::Reg)
                .unwrap_or(AllocationSpot::Stack());
            allocs.push(Allocation { liverange, loc });
        }
        allocs
    }

    fn insert_local_liverange(&mut self, vreg: VrReg, local: live::LocalLiveRange) {
        if let Some(liverange) = self.reg_to_liverange.get_mut(&vreg) {
            liverange.ranges.push(local);
        } else {
            self.reg_to_liverange.insert(
                vreg,
                LiveRange {
                    ranges: vec![local],
                    reg: vreg,
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
                let local_range = live::LocalLiveRange {
                    from: live::LiveLocation {
                        index: insns[0].id(),
                        mode: live::LiveLocationMode::Pre,
                    },
                    to: live::LiveLocation {
                        index: insns.last().unwrap().id(),
                        mode: live::LiveLocationMode::Post,
                    },
                };
                // TODO: Had to manually inline the insert_local_liverange function for borrow checker to be happy... Fix this later...
                let local_range_index = if let Some(liverange) = self.reg_to_liverange.get_mut(&reg)
                {
                    let index = liverange.ranges.len();
                    liverange.ranges.push(local_range);
                    index
                } else {
                    self.reg_to_liverange.insert(
                        *reg,
                        LiveRange {
                            ranges: vec![local_range],
                            reg: *reg,
                        },
                    );
                    0
                };

                // Set beginning of lifetime
                let entry =
                    &mut self.reg_to_liverange.get_mut(&reg).unwrap().ranges[local_range_index];
                let set_begin = !allocbb.live_in.contains(*reg);
                let set_end = !allocbb.live_out.contains(*reg);

                for ins in insns.iter().rev() {
                    if set_end {
                        for mode in &[live::LiveLocationMode::Pre, live::LiveLocationMode::Post] {
                            for operand in &ins.operands {
                                if operand.pos_mode != *mode
                                    || operand.reg != *reg
                                    || operand.rw_mode != RWMode::Use
                                {
                                    continue;
                                }

                                entry.to = live::LiveLocation {
                                    mode: *mode,
                                    index: ins.id(),
                                };
                            }
                        }
                    }

                    if set_begin {
                        for mode in &[live::LiveLocationMode::Post, live::LiveLocationMode::Pre] {
                            for operand in &ins.operands {
                                if operand.pos_mode != *mode
                                    || operand.reg != *reg
                                    || operand.rw_mode != RWMode::Def
                                {
                                    continue;
                                }

                                entry.from = live::LiveLocation {
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

#[cfg(test)]
mod tests {
    use crate::*;
    use gcf_bb as bb;
    use smallvec::smallvec;

    #[test]
    fn live_range() {
        let mut cfg = bb::ControlFlow::new();
        let bb0_real = bb::BasicBlock::new(smallvec![Instruction::new(
            InstructionId(0),
            vec![
                Operand {
                    id: OperandId(0),
                    fixation: Fixation::None,
                    pos_mode: live::LiveLocationMode::Pre,
                    reg: VrReg(0),
                    rw_mode: RWMode::Use,
                },
                Operand {
                    id: OperandId(1),
                    fixation: Fixation::None,
                    pos_mode: live::LiveLocationMode::Post,
                    reg: VrReg(1),
                    rw_mode: RWMode::Def,
                }
            ]
        )]);
        let bb1_real = bb::BasicBlock::new(smallvec![Instruction::new(
            InstructionId(1),
            vec![Operand {
                id: OperandId(0),
                fixation: Fixation::None,
                pos_mode: live::LiveLocationMode::Pre,
                reg: VrReg(1),
                rw_mode: RWMode::Use,
            }]
        )]);

        let bb0 = cfg.add_basic_block(bb0_real);
        let bb1 = cfg.add_basic_block(bb1_real);
        cfg.add_edge(bb0, bb1);

        let mut alloc = Allocator::new(cfg);
        alloc.block_inout();
        alloc.block_liverange();
        let allocations = alloc.allocate();
        dbg!(allocations);
    }
}
