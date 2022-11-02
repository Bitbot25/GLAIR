use glair::codegen;
use glair::compile;
use glair::rtl;
use glair::ssa;
use glair::typing;

use codegen::Codegen;
use compile::CompileIntoBlock;

fn main() {
    let x_0 = ssa::Variable::new("x", 0, typing::Type::U32);
    let y_0 = ssa::Variable::new("y", 1, typing::Type::U32);
    let y_1 = y_0.ssa_bump();

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
            ssa::Ins::Assign(
                x_0,
                ssa::RValue::Flat(ssa::FlatRValue::Lit(ssa::Literal::U32(10))),
            ),
            ssa::Ins::Assign(y_0, ssa::RValue::Flat(ssa::FlatRValue::Var(x_0))),
            ssa::Ins::Assign(
                y_1,
                ssa::RValue::BinOp(ssa::BinOp::Sub(
                    ssa::FlatRValue::Var(y_0),
                    ssa::FlatRValue::Var(x_0),
                )),
            ),
        ],
    };
    let compiled_rtl = bb.compile_into_block();
    let mut codegen_ctx = codegen::CodegenContext::default();
    codegen_ctx
        .pseudo_reg_mappings
        .insert(0, rtl::PhysRegister::Amd64(rtl::amd64::Amd64Register::Eax));
    codegen_ctx
        .pseudo_reg_mappings
        .insert(1, rtl::PhysRegister::Amd64(rtl::amd64::Amd64Register::Ecx));

    println!("{}", compiled_rtl);
    println!("{}", compiled_rtl.codegen_string(&mut codegen_ctx));

    /*let cp = rtl::Op::Copy(rtl::OpCopy {
        to: rtl::Register::Pseudo(0),
        from: rtl::RValue::Lit(rtl::Lit::LitU8(69)),
    });
    let add = rtl::Op::Add(rtl::OpAdd {
        to: rtl::Register::Pseudo(0),
        val: rtl::RValue::Lit(rtl::Lit::LitU8(1)),
    });
    let rtl = rtl::Block {
        name: Some("my_function".to_string()),
        ops: vec![cp, add],
        metadata: (),
    };*/
}
