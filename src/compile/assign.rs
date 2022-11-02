use crate::rtl;
use crate::ssa;
use crate::typing::Typed;

pub(super) fn compile(
    dest: &ssa::Variable,
    val: &ssa::RValue,
    ops: &mut rtl::Ops,
    context: &mut super::CompileContext,
) {
    assert_eq!(dest.typ(), val.typ());
    let dest_reg = context.acquire_variable_register(dest);
    match val {
        ssa::RValue::BinOp(op) => {
            compile_binop(dest_reg, op, ops, context);
        }
        ssa::RValue::Flat(val) => {
            let from = super::rtl_rvalue_from_flat_ssa(context, val);
            ops.push(rtl::Op::Copy(rtl::OpCopy { to: dest_reg, from }));
        }
    };
}

fn compile_binop(
    dest: rtl::Register,
    op: &ssa::BinOp,
    ops: &mut rtl::Ops,
    context: &mut super::CompileContext,
) {
    match op {
        ssa::BinOp::Sub(a, b) => {
            let a_rvalue = super::rtl_rvalue_from_flat_ssa(context, a);
            let b_rvalue = super::rtl_rvalue_from_flat_ssa(context, b);
            match a_rvalue {
                rtl::RValue::Register(reg) if dest == reg => (),
                _ => ops.push(rtl::Op::Copy(rtl::OpCopy {
                    to: dest,
                    from: a_rvalue,
                })),
            };
            ops.push(rtl::Op::Sub(rtl::OpSub {
                from: dest,
                val: b_rvalue,
            }));
        }
    }
}
