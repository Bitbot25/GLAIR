use crate::rtl;

pub mod amd64;

pub trait Codegen {
    fn codegen_string(&self, context: &mut CodegenContext) -> String;
}

#[derive(Default)]
pub struct CodegenContext;

pub fn unwrap_phys_register(
    opt: Option<&rtl::RealRegister>,
    n_pseudo: usize,
) -> &rtl::RealRegister {
    match opt {
        Some(x) => x,
        None => panic!("Pseudo register {} was not resolved.", n_pseudo),
    }
}

fn check_lvalue_rvalue(lvalue: &rtl::Register, rvalue: &rtl::RValue) {
    assert_eq!(
        lvalue.sz(),
        rvalue.sz(),
        "lvalue and rvalue are of the same size."
    );
}
