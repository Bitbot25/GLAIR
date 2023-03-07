mod live;

#[derive(Debug, Copy, Clone)]
pub struct InstructionId(pub usize);

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

        let mut alloc = live::Allocator::new(cfg);
        alloc.block_inout();
        alloc.block_liverange();
        dbg!(alloc);
    }
}
