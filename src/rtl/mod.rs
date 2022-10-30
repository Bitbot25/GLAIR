use std::collections::HashMap;

use crate::codegen;

pub mod amd64;
pub mod debug;

#[derive(Debug, Clone)]
pub enum PhysRegister {
    Amd64(amd64::Amd64Register),
    Amd64Memory(amd64::Amd64Memory),
}

impl PhysRegister {
    pub fn sz(&self) -> usize {
        match self {
            PhysRegister::Amd64(reg) => reg.sz(),
            PhysRegister::Amd64Memory(mem) => mem.sz(),
        }
    }
}

#[derive(Copy, Clone)]
pub struct Register(pub usize);

pub enum Lit {
    LitU8(u8),
    LitU32(u32),
}

impl Lit {
    pub fn sz(&self) -> usize {
        match self {
            Lit::LitU8(..) => codegen::BYTE_SZ,
            Lit::LitU32(..) => codegen::DWORD_SZ,
        }
    }
}

pub enum RValue {
    Register(Register),
    Lit(Lit),
}

impl RValue {
    pub fn sz(&self, pseudo_reg_mappings: &HashMap<usize, PhysRegister>) -> usize {
        match self {
            RValue::Register(reg) => {
                codegen::unwrap_phys_register(pseudo_reg_mappings.get(&reg.0), reg.0).sz()
            }
            RValue::Lit(lit) => lit.sz(),
        }
    }
}

pub struct OpCopy {
    pub to: Register,
    pub from: RValue,
}

pub struct OpSub {
    pub from: Register,
    pub val: RValue,
}

pub enum Op {
    Copy(OpCopy),
    Sub(OpSub),
}

pub type Ops = Vec<Op>;

pub struct Block {
    pub name: Option<String>,
    pub ops: Ops,
    pub metadata: (),
}
