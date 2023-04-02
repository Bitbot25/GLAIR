use gcf_bb as bb;
use smallvec::{smallvec, SmallVec};

mod display;
mod r#impl;
mod ifr;

#[derive(Debug)]
pub struct GAOperandNo(u32);
#[derive(Debug)]
pub struct GAVarNo(u32);
#[derive(Debug)]
pub struct GARegNo(u32);
#[derive(Debug)]
pub struct GAInstrNo(u32);
#[derive(Debug)]
pub struct GALocation {
    instr: GAInstrNo,
    when: GALiveWhen,
}

pub struct Allocno {
    constraints: SmallVec<[GAConstraint; 4]>,
    slice: LRSlice,
}

#[derive(Debug)]
pub struct GAInstr {
    operands: SmallVec<[GAOperand; 4]>,
    instrno: GAInstrNo,
}

#[derive(Debug)]
pub struct GAOperand {
    oprno: GAOperandNo,
    varno: GAVarNo,
    constraints: SmallVec<[GAConstraint; 2]>,
    when: GALiveWhen,
}

#[derive(Debug)]
pub enum GALiveWhen {
    Pre,
    Post,
}

#[derive(Debug)]
pub struct LRSlice {
    from: GALocation,
    to: GALocation,
}

#[derive(Debug)]
pub struct LiveRange {
    slices: SmallVec<[LRSlice; 4]>,
}

#[derive(Debug)]
pub enum GAConstraint {
    Reuse(GAOperandNo),
    Fix(GARegNo),
}

fn compute_postorder<I>(
    entry: bb::BasicBlockId,
    cfg: &bb::ControlFlow<I>,
) -> Vec<bb::BasicBlockId> {
    // Yanked straight from cranelift
    // TODO: Only compute successors and such once. Store it in a ControlFlowCache.
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

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test() {
        println!(
            "{}",
            LiveRange {
                slices: smallvec![LRSlice {
                    from: GALocation {
                        instr: GAInstrNo(0),
                        when: GALiveWhen::Pre
                    },
                    to: GALocation {
                        instr: GAInstrNo(0),
                        when: GALiveWhen::Post
                    }
                }]
            }
        );
    }
}
