use super::*;
use std::fmt::{Display, Formatter};

impl Display for amd64::Amd64Memory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            amd64::Amd64Memory::Register(sz, reg) => write!(f, "(amd64_reg {})", reg.name()),
            amd64::Amd64Memory::Addr(sz, addr) => write!(f, "(amd64_mem_addr {} {})", sz, addr),
            amd64::Amd64Memory::Add(operands) => write!(f, "(amd64_mem_add {} {})", operands.0, operands.1),
            amd64::Amd64Memory::Sub(operands) => write!(f, "(amd64_mem_sub {} {})", operands.0, operands.1),
        }
    }
}

impl Display for PhysRegister {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PhysRegister::Amd64(reg) => write!(f, "(reg_amd64 {}", reg.name()),
            PhysRegister::Amd64Memory(memory) => Display::fmt(memory, f),
        }
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(reg {})", self.0)
    }
}

impl Display for Lit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Lit::LitU8(val) => write!(f, "(lit_u8 {})", val),
            Lit::LitU32(val) => write!(f, "(lit_u32 {})", val),
        }
    }
}

impl Display for RValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RValue::Lit(lit) => Display::fmt(lit, f),
            RValue::Register(reg) => Display::fmt(reg, f),
        }
    }
}

impl Display for OpAdd {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(add {} {})", self.to, self.val)
    }
}

impl Display for OpCopy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(copy {} {})", self.to, self.from)
    }
}

impl Display for Op {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Op::Copy(copy) => Display::fmt(copy, f),
            Op::Add(add) => Display::fmt(add, f),
        }
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.name {
            Some(name) => write!(f, "# Block: '{}'\n", name)?,
            None => write!(f, "# Block\n")?,
        };
        write!(f, "(\n")?;
        for op in &self.ops {
            write!(f, "    {}\n", op)?;
        }
        write!(f, ")\n")?;
        Ok(())
    }
}
