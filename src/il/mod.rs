use std::hash::Hash;

pub mod cfg;
mod impl_amd;
mod impl_misc;

pub trait ILSized {
    fn il_size(&self) -> ILSize;
}

/// Immediate (literal) value.
#[derive(Debug)]
pub enum Immediate {
    U32(u32),
}

impl ILSized for Immediate {
    fn il_size(&self) -> ILSize {
        match self {
            Immediate::U32(_) => ILSize::Integer {
                width_in_bytes: 32 / 8,
            },
        }
    }
}

impl ILSized for burnerflame::Register {
    fn il_size(&self) -> ILSize {
        if self.is_64bit() {
            ILSize::Integer {
                width_in_bytes: 64 / 8,
            }
        } else if self.is_32bit() {
            ILSize::Integer {
                width_in_bytes: 32 / 8,
            }
        } else if self.is_16bit() {
            ILSize::Integer {
                width_in_bytes: 16 / 8,
            }
        } else if self.is_8bit() {
            ILSize::Integer { width_in_bytes: 1 }
        } else {
            panic!("Unknown size")
        }
    }
}

/// A SSARegister or Immediate value
#[derive(Debug)]
pub enum RValue {
    SSARegister(SSARegister),
    Immediate(Immediate),
}

#[derive(Debug, Copy, Clone)]
pub enum ILSize {
    Pointer,
    Integer { width_in_bytes: usize },
    Structure { width_in_bytes: usize },
}

impl ILSize {
    pub fn width(&self) -> usize {
        match self {
            ILSize::Pointer => {
                if cfg!(target_pointer_width = "64") {
                    64 / 8
                } else if cfg!(target_pointer_width = "32") {
                    32 / 8
                } else {
                    panic!("unsupported pointer width")
                }
            }
            ILSize::Integer { width_in_bytes } => *width_in_bytes,
            ILSize::Structure { width_in_bytes } => *width_in_bytes,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PlaceholderReg {
    pub identifier: usize,
    pub size: ILSize,
}

impl Hash for PlaceholderReg {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.identifier);
    }
}

impl Eq for PlaceholderReg {}

impl PartialEq for PlaceholderReg {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}

#[derive(Hash, Debug, Copy, Clone, PartialEq, Eq)]
pub enum MachineReg {
    AMD64(burnerflame::Register),
}

#[derive(Hash, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SSARegister {
    id: usize,
    machine_reg: Option<MachineReg>,
}

/// Allocates memory on the stack and places the pointer in [`out_pointer`]
#[derive(Debug)]
pub struct Reserve {
    pub size: ILSize,
    pub out_pointer: SSARegister,
}

/// Writes [`value`] to [`destination`]
#[derive(Debug)]
pub struct Write {
    pub destination: SSARegister,
    pub value: RValue,
}

/// Reads the data pointed to by [`target`] into [`out_data`]
#[derive(Debug)]
pub struct Read {
    pub target: SSARegister,
    pub out_data: SSARegister,
}

/// Returns the value found in [`register`]
#[derive(Debug)]
pub struct Return {
    pub register: Option<SSARegister>,
}

#[derive(Debug)]
pub struct DummyUse {
    pub register: SSARegister,
}

/// IL Instruction
#[derive(Debug)]
pub enum Instruction {
    Reserve(Reserve),
    Write(Write),
    DummyUse(DummyUse),
    Read(Read),
    Return(Return),
}
