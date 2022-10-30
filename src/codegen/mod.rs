use std::collections::HashMap;

use crate::rtl;

pub mod amd64;

pub const BYTE_SZ: usize = 1;
pub const WORD_SZ: usize = 2;
pub const DWORD_SZ: usize = 4;
pub const QWORD_SZ: usize = 8;

pub trait Codegen {
    fn codegen_string(&self, context: &mut CodegenContext) -> String;
}

#[derive(Default)]
pub struct CodegenContext {
    pub pseudo_reg_mappings: HashMap<usize, rtl::PhysRegister>,
}

pub fn unwrap_phys_register(opt: Option<&rtl::PhysRegister>, n_pseudo: usize) -> &rtl::PhysRegister {
    match opt {
        Some(x) => x,
        None => panic!("Pseudo register {} was not resolved.", n_pseudo),
    }
}

fn check_lvalue_rvalue(context: &mut CodegenContext, lvalue: &rtl::Register, rvalue: &rtl::RValue) {
    assert_eq!(
        unwrap_phys_register(context.pseudo_reg_mappings.get(&lvalue.0), lvalue.0).sz(),
        rvalue.sz(&context.pseudo_reg_mappings),
        "lvalue and rvalue are of the same type."
    );
}
