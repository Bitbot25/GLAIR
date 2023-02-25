use smallvec::{smallvec, SmallVec};
use std::collections::{HashMap, HashSet, VecDeque};

use gcf_bb as bb;

#[derive(Debug)]
pub struct InstructionId(pub usize);

pub struct LocalLiveRange {
    from: LiveLocation,
    to: LiveLocation,
}

pub struct LiveLocation {
    mode: LiveLocationMode,
    index: InstructionId,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LiveLocationMode {
    Pre,
    Post,
}

pub struct LiveRange {
    ranges: Vec<LocalLiveRange>,
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub struct VrReg(pub usize);
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub struct HwReg(pub usize);

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
    pub pos_mode: LiveLocationMode,
}

#[derive(Debug, Copy, Clone)]
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
}

#[derive(Debug)]
pub struct Allocator {
    blocks_postorder: Vec<bb::BasicBlockId>,
    block_to_allocbb: HashMap<bb::BasicBlockId, AllocBB>,
    cfg: bb::ControlFlow<Instruction>,
}

#[derive(Debug)]
struct AllocBB {
    bb: bb::BasicBlockId,
    live_in: LiveSet,
    live_out: LiveSet,
}

#[derive(Clone, Debug)]
struct LiveSet {
    vars: HashSet<VrReg>,
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
                },
            );
        }

        let me = Allocator {
            block_to_allocbb,
            blocks_postorder: compute_postorder(cfg.entry(), &cfg),
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

            let live_in = &mut self.block_to_allocbb.get_mut(&block_id).unwrap().live_in;

            for ins in block.instructions().iter().rev() {
                for mode in &[LiveLocationMode::Post, LiveLocationMode::Pre] {
                    for operand in &ins.operands {
                        if operand.pos_mode == *mode {
                            match operand.rw_mode {
                                RWMode::Def => live_in.set(operand.reg, false),
                                RWMode::Use => live_in.set(operand.reg, true),
                            }
                        }
                    }
                }
            }

            for pred in self.cfg.predecessors(block_id) {
                queue.push_back(pred);
            }
        }
    }
}
