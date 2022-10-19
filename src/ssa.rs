use super::typing::{self, Typed};
use std::fmt;

#[derive(Hash, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Variable {
    ver: usize,
    name: &'static str,
    typ: typing::Type,
}

impl Variable {
    pub fn new(name: &'static str, typ: typing::Type) -> Variable {
        Variable { name, ver: 0, typ }
    }

    pub fn ssa_bump(&self) -> Variable {
        Variable {
            name: self.name,
            ver: self.ver + 1,
            typ: self.typ,
        }
    }
}

impl typing::Typed for Variable {
    fn typ(&self) -> typing::Type {
        self.typ
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}_{}", self.name, self.ver)
    }
}

#[derive(Debug)]
pub enum InlineValue {
    I32(i32),
    U32(u32),
}

impl typing::Typed for InlineValue {
    fn typ(&self) -> typing::Type {
        match self {
            InlineValue::I32(..) => typing::Type::I32,
            InlineValue::U32(..) => typing::Type::U32,
        }
    }
}

impl fmt::Display for InlineValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InlineValue::I32(val) => write!(f, "{}", *val),
            InlineValue::U32(val) => write!(f, "{}", *val),
        }
    }
}

#[derive(Debug)]
pub enum Operand {
    Inline(InlineValue),
    Variable(Variable),
}

impl typing::Typed for Operand {
    #[inline]
    fn typ(&self) -> typing::Type {
        match self {
            Operand::Inline(inline) => inline.typ(),
            Operand::Variable(var) => var.typ(),
        }
    }
}

impl Operand {
    #[inline]
    pub fn mem_size(&self) -> usize {
        self.typ().mem_size()
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Inline(inline) => fmt::Display::fmt(inline, f),
            Operand::Variable(var) => fmt::Display::fmt(var, f),
        }
    }
}

#[derive(Debug)]
pub enum Ins {
    Sub(
        Variable,
        /* = */ Operand,
        /* - */ Operand,
    ),
    Init(Variable, /* <- */ Operand),
}
