mod glir;
mod rtl;

use glir::bb;
use glir::ssa;
use glir::typing;
use rtl::Codegen;

fn main() {
    let x_0 = ssa::Variable::new("x", typing::Type::I32);
    let x_1 = x_0.ssa_bump();

    let bb = bb::BasicBlock {
        terminator: bb::Terminator::Void,
        ins_list: vec![
            glir::Ins::Init(x_0, ssa::Operand::Inline(ssa::InlineValue::I32(10))),
            glir::Ins::Sub(
                x_1,
                ssa::Operand::Variable(x_0),
                ssa::Operand::Inline(ssa::InlineValue::I32(2)),
            ),
        ],
    };
    dbg!(&bb);

    let rtl = rtl::LBB {
        label: "LBB_0",
        ops: vec![
            rtl::Op::Move(
                rtl::Place::Sub(Box::new((
                    rtl::Place::Simple(rtl::SimplePlace::Register(rtl::REG_X86_ESP)),
                    rtl::Place::Simple(rtl::SimplePlace::Addr(rtl::WordTy::DWord, 4)),
                ))),
                rtl::Value::I32(10),
            ),
            rtl::Op::Sub(
                rtl::Place::Sub(Box::new((
                    rtl::Place::Simple(rtl::SimplePlace::Register(rtl::REG_X86_ESP)),
                    rtl::Place::Simple(rtl::SimplePlace::Addr(rtl::WordTy::DWord, 4)),
                ))),
                rtl::Value::I32(2),
            ),
        ],
    };
    println!("RTL: \n{}", rtl.nasm());
}
