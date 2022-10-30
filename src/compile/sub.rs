use crate::rtl;
use crate::ssa;
use crate::typing::Typed;

fn rtl_rvalue_from_ssa(context: &mut super::CompileContext, ssa: &ssa::RValue) -> rtl::RValue {
    match ssa {
        ssa::RValue::Lit(lit) => rtl::RValue::Lit(match lit {
            ssa::Literal::U32(val) => rtl::Lit::LitU32(*val),
            _ => todo!("Many literal types"),
        }),
        ssa::RValue::Variable(var) => rtl::RValue::Register(context.acquire_variable_register(var)),
    }
}

pub(super) fn compile(
    dest: &ssa::Variable,
    a: &ssa::RValue,
    b: &ssa::RValue,
    ops: &mut rtl::Ops,
    context: &mut super::CompileContext,
) {
    assert_eq!(dest.typ(), a.typ());
    assert_eq!(a.typ(), b.typ());

    let dest_reg = context.acquire_variable_register(dest);
    let a_reg = rtl_rvalue_from_ssa(context, a);
    let b_reg = rtl_rvalue_from_ssa(context, b);

    // This could be optimized to a lea instruction in amd64
    ops.push(rtl::Op::Copy(rtl::OpCopy {
        to: dest_reg,
        from: a_reg,
    }));
    ops.push(rtl::Op::Sub(rtl::OpSub {
        from: dest_reg,
        val: b_reg,
    }));
}
