use crate::reg::{AccessMode, Register};

#[derive(Debug, PartialEq, Eq)]
pub struct Template {
    pub id: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ImmediateExpr {
    Int32(i32),
    UInt32(u32),
    Template(Template)
}

impl ImmediateExpr {
    pub fn as_access_mode(&self) -> AccessMode {
        match self {
            ImmediateExpr::Int32(_) => AccessMode::SI,
            ImmediateExpr::UInt32(_) => AccessMode::SI,
            ImmediateExpr::Template(_) => panic!("Cannot convert template expr to accessmode"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Rtx {
    Destination(DestinationExpr),
    Immediate(ImmediateExpr),
}

#[derive(Debug, PartialEq, Eq)]
pub enum RtxRef<'a> {
    DestinationRef(&'a DestinationExpr),
    ImmediateRef(&'a ImmediateExpr),
}

#[derive(Debug, PartialEq, Eq)]
pub struct RegisterExpr {
    reg: Register,
    mode: AccessMode,
}

impl RegisterExpr {
    pub fn new(reg: Register, mode: AccessMode) -> Self {
        Self { reg, mode }
    }

    pub fn reg(&self) -> &Register {
        &self.reg
    }

    pub fn mode(&self) -> AccessMode {
        self.mode
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct MemoryExpr {
    pub deref: Box<Rtx>,
    pub mode: AccessMode,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DestinationExpr {
    Memory(MemoryExpr),
    Register(RegisterExpr),
    Template(Template),
}
