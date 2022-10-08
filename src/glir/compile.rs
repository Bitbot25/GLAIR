use std::collections::HashMap;

use super::*;
use crate::rtl;
use typing::Typed;

#[derive(Debug, Default)]
pub struct Compiler {
    pub variable_locations: HashMap<ssa::Variable, VariableLocation>,
    sp_inc: usize,
}

#[derive(Debug)]
pub enum VariableLocation {
    Stack { block_offset: usize },
    Register(rtl::Reg),
}

pub trait CompileIntoLBB {
    fn compile_into_bb(&self, compiler: &mut Compiler) -> rtl::LBB;
}

pub trait CompileIntoOps {
    fn compile_into_ops(&self, compiler: &mut Compiler, ops: &mut Vec<rtl::Op>);
}

impl CompileIntoLBB for bb::BasicBlock {
    fn compile_into_bb(&self, compiler: &mut Compiler) -> rtl::LBB {
        let mut ops = Vec::new();
        for ins in &self.ins_list {
            ins.compile_into_ops(compiler, &mut ops);
        }
        rtl::LBB { label: "BB_0", ops }
    }
}

impl CompileIntoOps for Ins {
    fn compile_into_ops(&self, compiler: &mut Compiler, ops: &mut Vec<rtl::Op>) {
        match self {
            Ins::Init(dest, val) => {
                assert_eq!(dest.typ(), val.typ());

                let location = compiler
                    .variable_locations
                    .get(dest)
                    .expect("Location of variable");
                let sz = dest.typ().mem_size();

                match location {
                    VariableLocation::Stack { block_offset } => {
                        ops.push(rtl::Op::Move(
                            rtl::Place::Deref(rtl::DerefPlace::Sub(Box::new((
                                rtl::DerefPlace::Reg(rtl::WordTy::DWord, rtl::REG_X86_ESP),
                                rtl::DerefPlace::Addr(
                                    rtl::WordTy::DWord,
                                    (block_offset - compiler.sp_inc) as isize,
                                ),
                            )))),
                            match val {
                                ssa::Operand::Inline(inline_val) => match inline_val {
                                    ssa::InlineValue::I32(num) => rtl::Value::I32(*num),
                                    ssa::InlineValue::U32(num) => rtl::Value::U32(*num),
                                },
                                ssa::Operand::Variable(var) => {
                                    match compiler.variable_locations.get(var) {
                                        Some(VariableLocation::Stack { block_offset }) => {
                                            if *block_offset == compiler.sp_inc {
                                                eprintln!("[INFO] Taking shortcut of not subtracting.");
                                                rtl::Value::Place(rtl::Place::Deref(rtl::DerefPlace::Reg(rtl::WordTy::DWord, rtl::REG_X86_ESP)))
                                            } else {
                                                todo!("calculate location of stack variable that's not on top of the stack")
                                            }
                                        },
                                        Some(VariableLocation::Register(_reg)) => todo!("variables in registers"),
                                        None => panic!("No location for variable!"),
                                    }
                                }
                            },
                        ));

                        // Decrement esp to match the top of the stack
                        ops.push(rtl::Op::Sub(
                            rtl::Place::Reg(rtl::REG_X86_ESP),
                            rtl::Value::U32((block_offset - compiler.sp_inc) as u32),
                        ));
                        compiler.sp_inc = *block_offset;
                    }
                    VariableLocation::Register(_reg) => todo!("Initializing of registers"),
                };
            }
            _ => todo!(),
        }
    }
}
