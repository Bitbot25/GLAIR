use crate::reg::{AccessMode, Register};

pub enum ImmediateExpr {
    Int32(i32),
    UInt32(u32),
}

impl ImmediateExpr {
    pub fn as_access_mode(&self) -> AccessMode {
        match self {
            ImmediateExpr::Int32(_) => AccessMode::SI,
            ImmediateExpr::UInt32(_) => AccessMode::SI,
        }
    }
}

pub enum Rtx {
    Destination(DestinationExpr),
    Immediate(ImmediateExpr),
}

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

pub struct MemoryExpr {
    deref: Box<Rtx>,
    mode: AccessMode,
}

pub enum DestinationExpr {
    Memory(MemoryExpr),
    Register(RegisterExpr),
}
