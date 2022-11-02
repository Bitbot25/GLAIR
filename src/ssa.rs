use super::typing::{self, Typed};
use std::{fmt, hash::Hash, hash::Hasher};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Variable {
    ver: usize,
    id: usize,
    name: &'static str,
    typ: typing::Type,
}

impl Hash for Variable {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        hasher.write_usize(self.id);
    }
}

impl Variable {
    pub fn new(name: &'static str, id: usize, typ: typing::Type) -> Variable {
        Variable {
            name,
            id,
            ver: 0,
            typ,
        }
    }

    pub fn ssa_bump(&self) -> Variable {
        Variable {
            name: self.name,
            typ: self.typ,
            id: self.id,
            ver: self.ver + 1,
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
pub enum BinOp {
    Sub(FlatRValue, FlatRValue),
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinOp::Sub(a, b) => write!(f, "(sub {} {})", a, b),
        }
    }
}

impl typing::Typed for BinOp {
    #[inline]
    fn typ(&self) -> typing::Type {
        match self {
            BinOp::Sub(a, b) => {
                assert_eq!(a.typ(), b.typ());
                a.typ()
            }
        }
    }
}

#[derive(Debug)]
pub enum FlatRValue {
    Lit(Literal),
    Var(Variable),
}

impl typing::Typed for FlatRValue {
    fn typ(&self) -> typing::Type {
        match self {
            FlatRValue::Lit(lit) => lit.typ(),
            FlatRValue::Var(var) => var.typ(),
        }
    }
}

impl fmt::Display for FlatRValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FlatRValue::Lit(lit) => fmt::Display::fmt(lit, f),
            FlatRValue::Var(var) => fmt::Display::fmt(var, f),
        }
    }
}

#[derive(Debug)]
pub enum RValue {
    BinOp(BinOp),
    Flat(FlatRValue),
}

impl typing::Typed for RValue {
    #[inline]
    fn typ(&self) -> typing::Type {
        match self {
            RValue::BinOp(op) => op.typ(),
            RValue::Flat(flat) => flat.typ(),
        }
    }
}

impl RValue {
    #[inline]
    pub fn mem_size(&self) -> usize {
        self.typ().mem_size()
    }
}

#[derive(Debug)]
pub enum Ins {
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
