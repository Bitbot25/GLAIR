mod compile;
mod rtl;
mod ssa;
mod typing;

use compile::CompileIntoLBB;
use rtl::Codegen;

fn main() {
    let x_0 = ssa::Variable::new("x", typing::Type::U32);
    let y_0 = ssa::Variable::new("y", typing::Type::U32);

    /*let bb = bb::BasicBlock {
        terminator: bb::Terminator::Void,
        ins_list: vec![
            glir::Ins::Init(x_0, ssa::Operand::Inline(ssa::InlineValue::I32(10))),
            glir::Ins::Sub(
                x_1,
                ssa::Operand::Variable(x_0),
                ssa::Operand::Inline(ssa::InlineValue::I32(2)),
            ),
        ],
    };*/
    let bb = ssa::BasicBlock {
        terminator: ssa::Terminator::Void,
        ins_list: vec![
            ssa::Ins::Assign(x_0, ssa::RValue::Lit(ssa::Literal::U32(10))),
            ssa::Ins::Assign(y_0, ssa::RValue::Variable(x_0)),
        ],
    };

    dbg!(&bb);
    let mut compiler = compile::Compiler::default();
    compiler
        .variable_locations
        .insert(x_0, compile::VariableLocation::Register(rtl::REG_X86_EAX));
    compiler
        .variable_locations
        .insert(y_0, compile::VariableLocation::Register(rtl::REG_X86_ECX));

    let rtl = bb.compile_into_bb(&mut compiler);
    eprintln!("RTL of GLIR:\n{:#?}", rtl);
    eprintln!("NASM of GLIR:\n{}", rtl.nasm());

    /*let _rtl = rtl::LBB {
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
    };*/
    // println!("RTL: \n{}", rtl.nasm());
}
