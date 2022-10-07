use super::*;
use crate::rtl;
use typing::Typed;

#[derive(Debug, Default)]
pub struct Compiler {
    sp: usize,
}

pub trait CompileIntoLBB {
    fn compile_into_bb(&self) -> rtl::LBB;
}

pub trait CompileIntoOps {
    fn compile_into_ops(&self, compiler: &mut Compiler, ops: &mut Vec<rtl::Op>);
}

impl CompileIntoLBB for bb::BasicBlock {
    fn compile_into_bb(&self) -> rtl::LBB {
        let mut ops = Vec::new();
        let mut compiler = Compiler::default();
        for ins in &self.ins_list {
            ins.compile_into_ops(&mut compiler, &mut ops);
        }
        rtl::LBB { label: "BB_0", ops }
    }
}

impl CompileIntoOps for Ins {
    fn compile_into_ops(&self, compiler: &mut Compiler, ops: &mut Vec<rtl::Op>) {
        match self {
            Ins::Init(dest, val) => {
                assert_eq!(dest.typ(), val.typ());

                // Move value
                ops.push(rtl::Op::Move(rtl::Place::Sub(Box::new((
                    rtl::Place::Simple(rtl::SimplePlace::Register(rtl::REG_X86_ESP)),
                    rtl::Place::Simple(rtl::SimplePlace::Addr(rtl::WordTy::DWord, dest.typ().mem_size() as isize)),
                ))), match val {
                    ssa::Operand::Inline(inline_val) => match inline_val {
                        ssa::InlineValue::I32(num) => rtl::Value::I32(*num),
                        ssa::InlineValue::U32(num) => rtl::Value::U32(*num),
                    },
                    ssa::Operand::Variable(_var) => todo!(),
                }));

                // Decrement esp
                ops.push(rtl::Op::Sub(rtl::Place::Simple(rtl::SimplePlace::Register(rtl::REG_X86_ESP)), rtl::Value::U32(dest.typ().mem_size() as u32)));

                compiler.sp += dest.typ().mem_size();
            }
            _ => todo!(),
        }
    }
}
