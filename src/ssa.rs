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
pub enum Literal {
    I32(i32),
    U32(u32),
}

impl typing::Typed for Literal {
    fn typ(&self) -> typing::Type {
        match self {
            Literal::I32(..) => typing::Type::I32,
            Literal::U32(..) => typing::Type::U32,
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::I32(val) => write!(f, "{}", *val),
            Literal::U32(val) => write!(f, "{}", *val),
        }
    }
}

#[derive(Debug)]
pub enum RValue {
    Lit(Literal),
    Variable(Variable),
}

impl typing::Typed for RValue {
    #[inline]
    fn typ(&self) -> typing::Type {
        match self {
            RValue::Lit(inline) => inline.typ(),
            RValue::Variable(var) => var.typ(),
        }
    }
}

impl RValue {
    #[inline]
    pub fn mem_size(&self) -> usize {
        self.typ().mem_size()
    }
}

impl fmt::Display for RValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RValue::Lit(inline) => fmt::Display::fmt(inline, f),
            RValue::Variable(var) => fmt::Display::fmt(var, f),
        }
    }
}

#[derive(Debug)]
pub enum Ins {
    Sub(Variable, /* = */ RValue, /* - */ RValue),
    Assign(Variable, /* <- */ RValue),
}

#[derive(Debug)]
pub struct BasicBlock {
    pub ins_list: Vec<Ins>,
    pub terminator: Terminator,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum Terminator {
    Ret(Variable),
    Jmp(Box<BasicBlock>),
    Void,
}
