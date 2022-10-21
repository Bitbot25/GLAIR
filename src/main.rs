mod codegen;
mod rtl;
mod ssa;
mod typing;
mod compile;

use compile::CompileIntoBlock;
use codegen::Codegen;
use codegen::DWORD_SZ;
// use rtl::Codegen;

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
            ssa::Ins::Init(x_0, ssa::Operand::Inline(ssa::InlineValue::U32(10))),
            ssa::Ins::Init(y_0, ssa::Operand::Variable(x_0)),
        ],
    };
    let compiled_rtl = bb.compile_into_block();
    let mut codegen_ctx = codegen::CodegenContext::default();
    codegen_ctx.pseudo_reg_mappings.insert(0, rtl::PhysRegister::Amd64(rtl::amd64::Amd64Register::Eax));
    codegen_ctx.pseudo_reg_mappings.insert(1, rtl::PhysRegister::Amd64Memory(rtl::amd64::Amd64Memory::Register(DWORD_SZ, rtl::amd64::Amd64Register::Ecx)));

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
