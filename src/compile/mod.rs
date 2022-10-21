mod init;

use std::collections::HashMap;
use crate::ssa;
use crate::rtl;

#[derive(Debug, Default)]
pub struct Compiler {
    pub variable_locations: HashMap<ssa::Variable, VariableLocation>,
    sp_inc: usize,
}

#[derive(Debug)]
pub enum VariableLocation {
    Stack { block_offset: usize },
    Register(rtl::Reg),
}

pub trait CompileIntoLBB {
    fn compile_into_bb(&self, compiler: &mut Compiler) -> rtl::LBB;
}

pub trait CompileIntoOps {
    fn compile_into_ops(&self, compiler: &mut Compiler, ops: &mut Vec<rtl::Op>);
}

impl CompileIntoLBB for ssa::BasicBlock {
    fn compile_into_bb(&self, compiler: &mut Compiler) -> rtl::LBB {
        let mut ops = Vec::new();
        for ins in &self.ins_list {
            ins.compile_into_ops(compiler, &mut ops);
        }
        rtl::LBB { label: "BB_0", ops }
    }
}

impl CompileIntoOps for ssa::Ins {
    fn compile_into_ops(&self, compiler: &mut Compiler, ops: &mut Vec<rtl::Op>) {
        match self {
            ssa::Ins::Init(dest, val) => init::compile(dest, val, compiler, ops),
            _ => todo!(),
        }
    }
}
