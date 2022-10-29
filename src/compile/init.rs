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
    let from = match val {
        ssa::RValue::Lit(inline) => rtl::RValue::Lit(match inline {
            ssa::Literal::U32(val) => rtl::Lit::LitU32(*val),
            _ => todo!(),
        }),
        ssa::RValue::Variable(variable) => {
            rtl::RValue::Register(context.acquire_variable_register(variable))
        }
    };
    ops.push(rtl::Op::Copy(rtl::OpCopy { to: dest_reg, from }));
}
