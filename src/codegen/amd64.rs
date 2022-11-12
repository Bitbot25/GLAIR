use super::{Codegen, CodegenContext};
use crate::rtl;

impl Codegen for rtl::amd64::Amd64Register {
    fn codegen_string(&self, _context: &mut CodegenContext) -> String {
        self.name().to_string()
    }
}

impl Codegen for rtl::Lit {
    fn codegen_string(&self, _context: &mut CodegenContext) -> String {
        match self {
            rtl::Lit::LitU8(val) => val.to_string(),
            rtl::Lit::LitU32(val) => val.to_string(),
        }
    }
}

impl Codegen for rtl::RealRegister {
    fn codegen_string(&self, context: &mut CodegenContext) -> String {
        match self {
            rtl::RealRegister::Amd64(reg) => reg.codegen_string(context),
        }
    }
}

impl Codegen for rtl::Register {
    fn codegen_string(&self, context: &mut CodegenContext) -> String {
        self.unwrap_real().codegen_string(context)
    }
}

impl Codegen for rtl::RValue {
    fn codegen_string(&self, context: &mut CodegenContext) -> String {
        match self {
            rtl::RValue::Lit(lit) => lit.codegen_string(context),
            rtl::RValue::Register(reg) => reg.codegen_string(context),
        }
    }
}

impl Codegen for rtl::Op {
    fn codegen_string(&self, context: &mut CodegenContext) -> String {
        match self {
            rtl::Op::Copy(cp) => {
                // Check that both operands are of the same size.
                super::check_lvalue_rvalue(&cp.to, &cp.from);
                format!(
                    "mov {}, {}",
                    cp.to.codegen_string(context),
                    cp.from.codegen_string(context)
                )
            }
            rtl::Op::Add(add) => {
                super::check_lvalue_rvalue(&add.to, &add.val);
                format!(
                    "add {}, {}",
                    add.to.codegen_string(context),
                    add.val.codegen_string(context)
                )
            }
            rtl::Op::Sub(sub) => {
                // Check that both operands are of the same size.
                super::check_lvalue_rvalue(&sub.from, &sub.val);
                format!(
                    "sub {}, {}",
                    sub.from.codegen_string(context),
                    sub.val.codegen_string(context)
                )
            }
            rtl::Op::Mul(..) => todo!("codegen amd64 nasm for mul"),
            rtl::Op::Div(..) => todo!("codegen amd64 nasm for div"),
        }
    }
}

impl Codegen for rtl::Block {
    fn codegen_string(&self, context: &mut CodegenContext) -> String {
        let mut buf = ";; ".to_string();
        match &self.name {
            Some(name) => buf.push_str(name.as_str()),
            None => buf.push_str("<unnamed block>"),
        };
        buf.push('\n');

        for op in &self.ops {
            buf.push_str(op.codegen_string(context).as_str());
            buf.push('\n');
        }

        buf
    }
}
