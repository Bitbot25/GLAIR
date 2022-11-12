use crate::rtl;
use crate::ssa;
use crate::typing::Typed;

/*pub(super) fn compile(
    dest: &ssa::Variable,
    val: &ssa::RValue,
    ops: &mut rtl::Ops,
    context: &mut super::CompileContext,
) {
    assert_eq!(dest.data_ty(), val.data_ty());
    let dest_reg = context.acquire_variable_register(dest);
    match val {
        ssa::RValue::BinOp(op) => {
            compile_binop(dest_reg, op, ops, context);
        }
        ssa::RValue::Opr(opr) => {
            let from = super::rtl_rvalue_from_ssa(context, opr);
            ops.push(rtl::Op::Copy(rtl::OpCopy { to: dest_reg, from }));
        }
    };
}*/

pub fn compile(
    dest: &ssa::Variable,
    a: &ssa::RValue,
    b: &ssa::RValue,
    binop_ty: ssa::BinOpTy,
    ops: &mut rtl::Ops,
) {
    assert_eq!(a.data_ty(), b.data_ty(), "operand types are equal");
    assert_eq!(
        dest.data_ty(),
        a.data_ty(),
        "dest and operand types are equal"
    );
    let a_rv = super::rtl_rvalue_from_ssa(a);
    let b_rv = super::rtl_rvalue_from_ssa(b);
    let dest_reg = rtl::Register::Vir(dest.as_vir_reg());
    match a_rv {
        rtl::RValue::Register(reg) if dest_reg == reg => (),
        _ => ops.push(rtl::Op::Copy(rtl::OpCopy {
            to: dest_reg,
            from: a_rv,
        })),
    };
    ops.push(match binop_ty {
        ssa::BinOpTy::Add => rtl::Op::Add(rtl::OpAdd {
            to: dest_reg,
            val: b_rv,
        }),
        ssa::BinOpTy::Sub => rtl::Op::Sub(rtl::OpSub {
            from: dest_reg,
            val: b_rv,
        }),
        ssa::BinOpTy::Mul => rtl::Op::Mul(rtl::OpMul {
            val: dest_reg,
            with: b_rv,
        }),
        ssa::BinOpTy::Div => rtl::Op::Div(rtl::OpDiv {
            val: dest_reg,
            with: b_rv,
        }),
    })
}

/*fn compile_binop(
    dest: rtl::Register,
    op: &ssa::BinOp,
    ops: &mut rtl::Ops,
    context: &mut super::CompileContext,
) {
    let a_rvalue = super::rtl_rvalue_from_ssa(context, &op.a);
    let b_rvalue = super::rtl_rvalue_from_ssa(context, &op.b);
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
}*/
