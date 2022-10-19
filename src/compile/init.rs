use crate::rtl;
use crate::ssa;
use crate::typing::Typed;
use crate::rtl::AsWordTy;
use super::{Compiler, VariableLocation};

pub(super) fn compile(
    dest: &ssa::Variable,
    val: &ssa::Operand,
    compiler: &mut Compiler,
    ops: &mut Vec<rtl::Op>,
) {
    assert_eq!(dest.typ(), val.typ());

    let location = compiler
        .variable_locations
        .get(dest)
        .expect("Location of variable");
    let g_wty = dest.typ().word_ty();
    match location {
        VariableLocation::Stack { block_offset } => {
            ops.push(rtl::Op::Move(
                rtl::Place::Expr(
                    g_wty,
                    rtl::PlaceExpr::Sub(Box::new((
                        rtl::PlaceExpr::Reg(rtl::REG_X86_ESP),
                        rtl::PlaceExpr::Addr((block_offset - compiler.sp_inc) as isize),
                    ))),
                ),
                match val {
                    ssa::Operand::Inline(inline_val) => match inline_val {
                        ssa::InlineValue::I32(num) => rtl::Value::I32(*num),
                        ssa::InlineValue::U32(num) => rtl::Value::U32(*num),
                    },
                    ssa::Operand::Variable(var) => match compiler.variable_locations.get(var) {
                        Some(VariableLocation::Stack { block_offset }) => {
                            let diff = *block_offset as isize - compiler.sp_inc as isize;
                            match diff {
                                0 => rtl::Value::Place(rtl::Place::Expr(
                                    g_wty,
                                    rtl::PlaceExpr::Reg(rtl::REG_X86_ESP),
                                )),
                                _ => rtl::Value::Place(rtl::Place::Expr(
                                    g_wty,
                                    rtl::PlaceExpr::Sub(Box::new((
                                        rtl::PlaceExpr::Reg(rtl::REG_X86_EDX),
                                        rtl::PlaceExpr::Addr(diff),
                                    ))),
                                )),
                            }
                        }
                        Some(VariableLocation::Register(reg)) => {
                            rtl::Value::Place(rtl::Place::Reg(*reg))
                        }
                        None => panic!("No location for variable!"),
                    },
                },
            ));

            // Set esp to match the top of the stack
            compiler.sp_inc = compiler.sp_inc.max(*block_offset);
            ops.push(rtl::Op::Move(
                rtl::Place::Reg(rtl::REG_X86_ESP),
                rtl::Value::U32(compiler.sp_inc as u32),
            ));
        }
        VariableLocation::Register(reg) => {
            ops.push(rtl::Op::Move(
                rtl::Place::Reg(*reg),
                match val {
                    ssa::Operand::Inline(ssa::InlineValue::I32(num)) => rtl::Value::I32(*num),
                    ssa::Operand::Inline(ssa::InlineValue::U32(num)) => rtl::Value::U32(*num),
                    ssa::Operand::Variable(var) => match compiler.variable_locations.get(var) {
                        Some(VariableLocation::Register(var_reg)) => {
                            rtl::Value::Place(rtl::Place::Reg(*var_reg))
                        }
                        Some(VariableLocation::Stack { block_offset }) => {
                            let diff = *block_offset as isize - compiler.sp_inc as isize;
                            let wrd = reg.word_ty();
                            match diff {
                                0 => rtl::Value::Place(rtl::Place::Expr(
                                    wrd,
                                    rtl::PlaceExpr::Reg(rtl::REG_X86_ESP),
                                )),
                                _ => rtl::Value::Place(rtl::Place::Expr(
                                    wrd,
                                    rtl::PlaceExpr::Sub(Box::new((
                                        rtl::PlaceExpr::Reg(rtl::REG_X86_ESP),
                                        rtl::PlaceExpr::Addr(diff),
                                    ))),
                                )),
                            }
                        }
                        None => panic!("Not location for variable!"),
                    },
                },
            ));
        }
    };
}
