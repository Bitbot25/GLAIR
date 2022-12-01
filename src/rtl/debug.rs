use super::*;
use std::fmt::{Display, Formatter};

impl Display for RealRegister {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RealRegister::Amd64(reg) => write!(f, "(reg_amd64 {}", reg.name()),
        }
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Register::Vir(VirRegister { n, bytes }) => {
                write!(f, "(reg:{bytes} {n})")
            }
            Register::Stack(StackRegister { slot, bytes }) => {
                write!(f, "(stack:{bytes} {slot})")
            }
            Register::Real(reg) => Display::fmt(reg, f),
        }
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

impl Display for OpSub {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(sub {} {})", self.from, self.val)
    }
}

impl Display for OpMul {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(mul {} {})", self.val, self.with)
    }
}

impl Display for OpDiv {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(div {} {})", self.val, self.with)
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
            Op::Sub(sub) => Display::fmt(sub, f),
            Op::Mul(mul) => Display::fmt(mul, f),
            Op::Div(div) => Display::fmt(div, f),
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
