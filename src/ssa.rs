use crate::rtl;

use super::typing::{self, Typed};
use std::{fmt, hash::Hash, hash::Hasher};

#[derive(Default)]
pub struct GLIRSupervisor {
    variables: Vec<Variable>,
}

impl GLIRSupervisor {
    pub fn new() -> GLIRSupervisor {
        Self::default()
    }

    pub fn create_var(&mut self, ty: typing::Type) -> Variable {
        let v = Variable {
            id: self.variables.len(),
            ty,
        };
        self.variables.push(v);
        v
    }

    pub fn create_descendant(&mut self, v: Variable) -> Variable {
        assert!(self.variables.contains(&v));
        Variable {
            id: self.variables.len(),
            ty: v.data_ty(),
        }
    }

    pub fn vars(&self) -> &Vec<Variable> {
        &self.variables
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Variable {
    pub(self) id: usize,
    pub(self) ty: typing::Type,
}

impl Hash for Variable {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        hasher.write_usize(self.id);
    }
}

impl Variable {
    /*
    pub fn ssa_bump(&self) -> Variable {
        Variable {
            name: self.name,
            typ: self.typ,
            id: self.id,
        }
    }*/

    pub fn as_vir_reg(&self) -> rtl::VirRegister {
        rtl::VirRegister {
            bytes: self.data_ty().mem_size(),
            n: self.id,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }
}

impl typing::Typed for Variable {
    fn data_ty(&self) -> typing::Type {
        self.ty
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.id)
    }
}

pub enum BinOpTy {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Copy, Clone, Debug)]
pub enum Literal {
    I32(i32),
    U32(u32),
}

impl typing::Typed for Literal {
    fn data_ty(&self) -> typing::Type {
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
    Var(Variable),
    Lit(Literal),
}

impl typing::Typed for RValue {
    fn data_ty(&self) -> typing::Type {
        match self {
            RValue::Lit(lit) => lit.data_ty(),
            RValue::Var(var) => var.data_ty(),
        }
    }
}

impl fmt::Display for RValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RValue::Lit(lit) => fmt::Display::fmt(lit, f),
            RValue::Var(var) => fmt::Display::fmt(var, f),
        }
    }
}

#[derive(Debug)]
pub enum Ins {
    Add(Variable, /* = */ RValue, /* + */ RValue),
    Sub(Variable, /* = */ RValue, /* - */ RValue),
    Mul(Variable, /* = */ RValue, /* * */ RValue),
    Div(Variable, /* = */ RValue, /* / */ RValue),
    Cpy(Variable, /* = */ RValue),
}

#[derive(Default, Debug)]
pub struct BasicBlock {
    pub(crate) ins_list: Vec<Ins>,
    // pub(self) terminator: Terminator,
}

impl BasicBlock {
    pub fn new() -> BasicBlock {
        Self::default()
    }

    pub fn emitter<'a>(&'a mut self) -> BasicBlockEmitter<'a> {
        BasicBlockEmitter { bb: self }
    }
}

pub struct BasicBlockEmitter<'bb> {
    bb: &'bb mut BasicBlock,
}

impl<'bb> BasicBlockEmitter<'bb> {
    pub fn emit_cpy(&mut self, lhs: Variable, rhs: RValue) {
        self.bb.ins_list.push(Ins::Cpy(lhs, rhs));
    }

    pub fn emit_lit(&mut self, lhs: Variable, lit: Literal) {
        self.emit_cpy(lhs, RValue::Lit(lit));
    }

    pub fn emit_binop(&mut self, lhs: Variable, a: RValue, b: RValue, ty: BinOpTy) {
        self.bb.ins_list.push(match ty {
            BinOpTy::Add => Ins::Add(lhs, a, b),
            BinOpTy::Sub => Ins::Sub(lhs, a, b),
            BinOpTy::Mul => Ins::Mul(lhs, a, b),
            BinOpTy::Div => Ins::Div(lhs, a, b),
        })
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum Terminator {
    Ret(Variable),
    Jmp(Box<BasicBlock>),
    Void,
}
