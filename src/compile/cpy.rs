use crate::rtl;
use crate::ssa;
use crate::typing::Typed;

pub fn compile(dest: &ssa::Variable, rhs: &ssa::RValue, ops: &mut rtl::Ops) {
    assert_eq!(
        dest.data_ty(),
        rhs.data_ty(),
        "lhs and rhs have the same type."
    );
    ops.push(rtl::Op::Copy(rtl::OpCopy {
        to: rtl::Register::Vir(dest.as_vir_reg()),
        from: super::rtl_rvalue_from_ssa(rhs),
    }));
}
