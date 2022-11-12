mod binop;
mod cpy;
pub mod ralloc;

use crate::rtl;
use crate::ssa;

#[derive(Default)]
pub struct CompileContext;

pub trait CompileIntoBlock {
    fn compile_into_block(&self) -> rtl::Block;
}

pub trait CompileIntoOps {
    fn compile_into_ops(&self, ops: &mut rtl::Ops, context: &mut CompileContext);
}

impl CompileIntoBlock for ssa::BasicBlock {
    fn compile_into_block(&self) -> rtl::Block {
        let mut context = CompileContext::default();
        let mut ops = rtl::Ops::new();
        for ins in &self.ins_list {
            ins.compile_into_ops(&mut ops, &mut context);
        }
        rtl::Block {
            metadata: (),
            ops,
            name: Some("LBB_0".to_string()),
        }
    }
}

impl CompileIntoOps for ssa::Ins {
    fn compile_into_ops(&self, ops: &mut Vec<rtl::Op>, _context: &mut CompileContext) {
        match self {
            ssa::Ins::Add(dest, a, b) => binop::compile(dest, a, b, ssa::BinOpTy::Add, ops),
            ssa::Ins::Sub(dest, a, b) => binop::compile(dest, a, b, ssa::BinOpTy::Sub, ops),
            ssa::Ins::Mul(dest, a, b) => binop::compile(dest, a, b, ssa::BinOpTy::Mul, ops),
            ssa::Ins::Div(dest, a, b) => binop::compile(dest, a, b, ssa::BinOpTy::Div, ops),
            ssa::Ins::Cpy(dest, rhs) => cpy::compile(dest, rhs, ops),
        }
    }
}

#[inline]
fn rtl_rvalue_from_ssa(ssa: &ssa::RValue) -> rtl::RValue {
    match ssa {
        ssa::RValue::Lit(lit) => rtl::RValue::Lit(match lit {
            ssa::Literal::U32(val) => rtl::Lit::LitU32(*val),
            _ => todo!("Many literal types"),
        }),
        ssa::RValue::Var(var) => rtl::RValue::Register(rtl::Register::Vir(var.as_vir_reg())),
    }
}
