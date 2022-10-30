use crate::rtl;
use crate::ssa;
use crate::typing::Typed;

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
    let a_reg = super::rtl_rvalue_from_ssa(context, a);
    let b_reg = super::rtl_rvalue_from_ssa(context, b);

    // TODO: This could be optimized to a lea instruction in amd64
    // TODO: Implement optimization layers instead if this?

    match a_reg {
        rtl::RValue::Register(reg) if dest_reg == reg => (),
        _ => ops.push(rtl::Op::Copy(rtl::OpCopy {
            to: dest_reg,
            from: a_reg,
        })),
    };
    ops.push(rtl::Op::Sub(rtl::OpSub {
        from: dest_reg,
        val: b_reg,
    }));
}
