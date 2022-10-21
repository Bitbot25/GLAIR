use super::{Codegen, CodegenContext, BYTE_SZ, WORD_SZ, DWORD_SZ, QWORD_SZ};
use crate::rtl::{self, PhysRegister};

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

impl Codegen for rtl::PhysRegister {
    fn codegen_string(&self, context: &mut CodegenContext) -> String {
        match self {
            rtl::PhysRegister::Amd64(reg) => reg.codegen_string(context),
            rtl::PhysRegister::Amd64Memory(mem) => format!(
                "{} [{}]",
                match mem.sz() {
                    BYTE_SZ => "byte",
                    WORD_SZ => "word",
                    DWORD_SZ => "dword",
                    QWORD_SZ => "qword",
                    _ => panic!("unworded size!"),
                },
                mem.codegen_string(context)
            ),
        }
    }
}

impl Codegen for rtl::amd64::Amd64Memory {
    fn codegen_string(&self, context: &mut CodegenContext) -> String {
        match self {
            rtl::amd64::Amd64Memory::Register(_sz, reg) => reg.codegen_string(context),
            rtl::amd64::Amd64Memory::Addr(_sz, addr) => addr.to_string(),
            rtl::amd64::Amd64Memory::Add(operands) => format!(
                "{}+{}",
                operands.0.codegen_string(context),
                operands.1.codegen_string(context)
            ),
            rtl::amd64::Amd64Memory::Sub(operands) => format!(
                "{}-{}",
                operands.0.codegen_string(context),
                operands.1.codegen_string(context)
            ),
        }
    }
}


impl Codegen for rtl::Register {
    fn codegen_string(&self, context: &mut CodegenContext) -> String {
        // FIXME: Do i really need to clone here? Is there a better way?:
        let phys_reg = super::unwrap_phys_register(context.pseudo_reg_mappings.get(&self.0), self.0).clone();
        phys_reg.codegen_string(context)
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
                super::check_lvalue_rvalue(context, &cp.to, &cp.from);
                format!(
                    "mov {}, {}",
                    cp.to.codegen_string(context),
                    cp.from.codegen_string(context)
                )
            }
            rtl::Op::Add(add) => {
                // Check that both operands are of the same size.
                super::check_lvalue_rvalue(context, &add.to, &add.val);
                format!(
                    "add {}, {}",
                    add.to.codegen_string(context),
                    add.val.codegen_string(context)
                )
            }
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
