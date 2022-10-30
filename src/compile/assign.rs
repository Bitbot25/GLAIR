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
    let from = super::rtl_rvalue_from_ssa(context, val);
    ops.push(rtl::Op::Copy(rtl::OpCopy { to: dest_reg, from }));
}
