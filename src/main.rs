mod glir;
mod rtl;

use rtl::Codegen;
use glir::ssa;
use glir::bb;
use glir::typing;

fn main() {
    let x_0 = ssa::Variable::new("x", typing::Type::I32);
    let x_1 = x_0.ssa_bump();

    let _bb = bb::BasicBlock {
        terminator: bb::Terminator::Void,
        ins_list: vec![
            glir::Ins::Init(x_0, ssa::Operand::Inline(ssa::InlineValue::I32(10))),
            glir::Ins::Sub(
                x_1,
                ssa::Operand::Variable(x_0),
                ssa::Operand::Inline(ssa::InlineValue::I32(5)),
            ),
        ],
    };
    // println!("block: {:?}", bb);

    let rtl = rtl::LBB {
        label: "LBB_0",
        ops: vec![rtl::Op::Move(rtl::Place::Stack { typ: rtl::StackType::DWord, sp_offset: 4 }, rtl::Value::I32(10))],
    };
    println!("RTL: \n{}", rtl.nasm());

}
