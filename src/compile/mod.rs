mod assign;

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::Hash;
use std::hash::Hasher;

use crate::rtl;
use crate::ssa;

#[derive(Default)]
pub struct CompileContext {
    pub(self) reg_number: usize,
    // u64 is the hash of the ssa::Variable
    pub(self) registers: HashMap<u64, rtl::Register>,
}

impl CompileContext {
    fn next_pseudo_reg(&mut self) -> usize {
        self.reg_number += 1;
        self.reg_number - 1
    }

    pub fn acquire_variable_register(&mut self, var: &ssa::Variable) -> rtl::Register {
        let mut hasher = DefaultHasher::new();
        var.hash(&mut hasher);
        let hash_of_var = hasher.finish();
        let val = self.registers.get(&hash_of_var);

        match val {
            Some(reg) => *reg,
            None => {
                let reg = rtl::Register(self.next_pseudo_reg());
                self.registers.insert(hash_of_var, reg);
                reg
            }
        }
    }
}

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
    fn compile_into_ops(&self, ops: &mut Vec<rtl::Op>, context: &mut CompileContext) {
        match self {
            ssa::Ins::Assign(dest, val) => assign::compile(dest, val, ops, context),
        }
    }
}

#[inline]
fn rtl_rvalue_from_flat_ssa(context: &mut CompileContext, ssa: &ssa::FlatRValue) -> rtl::RValue {
    match ssa {
        ssa::FlatRValue::Lit(lit) => rtl::RValue::Lit(match lit {
            ssa::Literal::U32(val) => rtl::Lit::LitU32(*val),
            _ => todo!("Many literal types"),
        }),
        ssa::FlatRValue::Var(var) => rtl::RValue::Register(context.acquire_variable_register(var)),
    }
}
