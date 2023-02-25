mod live;

#[cfg(test)]
mod tests {
    use crate::live;
    use gcf_bb as bb;
    use smallvec::smallvec;

    #[test]
    fn live_range() {
        let mut cfg = bb::ControlFlow::new();
        let bb0_real = bb::BasicBlock::new(smallvec![live::Instruction::new(
            live::InstructionId(0),
            vec![
                live::Operand {
                    id: live::OperandId(0),
                    fixation: live::Fixation::None,
                    pos_mode: live::LiveLocationMode::Pre,
                    reg: live::VrReg(0),
                    rw_mode: live::RWMode::Use,
                },
                live::Operand {
                    id: live::OperandId(1),
                    fixation: live::Fixation::None,
                    pos_mode: live::LiveLocationMode::Post,
                    reg: live::VrReg(1),
                    rw_mode: live::RWMode::Def,
                }
            ]
        )]);
        let bb1_real = bb::BasicBlock::new(smallvec![live::Instruction::new(
            live::InstructionId(1),
            vec![live::Operand {
                id: live::OperandId(0),
                fixation: live::Fixation::None,
                pos_mode: live::LiveLocationMode::Pre,
                reg: live::VrReg(1),
                rw_mode: live::RWMode::Use,
            }]
        )]);
        let bb2_real = bb::BasicBlock::new(smallvec![live::Instruction::new(
            live::InstructionId(1),
            vec![live::Operand {
                id: live::OperandId(0),
                fixation: live::Fixation::None,
                pos_mode: live::LiveLocationMode::Pre,
                reg: live::VrReg(1),
                rw_mode: live::RWMode::Use,
            }]
        )]);
        let bb0 = cfg.add_basic_block(bb0_real);
        let bb1 = cfg.add_basic_block(bb1_real);
        cfg.add_edge(bb0, bb1);

        let mut alloc = live::Allocator::new(cfg);
        alloc.block_inout();
        dbg!(alloc);
    }
}
