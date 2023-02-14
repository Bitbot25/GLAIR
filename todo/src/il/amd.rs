use std::fmt::{self, Debug};

use super::{ILSize, ILSized};

#[derive(PartialEq, Eq, Debug)]
pub enum AmdRegUnit {
    AL,
    CL,
    BL,
    DL,

    AH,
    CH,
    BH,
    DH,
}

pub struct AmdRegSpec {
    libmc: burnerflame::Register,
    units: &'static [AmdRegUnit],
}

static RAX: AmdRegSpec = AmdRegSpec {
    libmc: burnerflame::Register::RAX,
    units: &[AmdRegUnit::AL, AmdRegUnit::AH],
};
static EAX: AmdRegSpec = AmdRegSpec {
    units: &[AmdRegUnit::AL, AmdRegUnit::AH],
    libmc: burnerflame::Register::EAX,
};
static ECX: AmdRegSpec = AmdRegSpec {
    units: &[AmdRegUnit::CL, AmdRegUnit::CH],
    libmc: burnerflame::Register::ECX,
};

impl PartialEq for AmdRegister {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0 as *const _, other.0 as *const _)
    }
}

impl Eq for AmdRegister {}

impl Debug for AmdRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} -> {:?}", self.0.libmc, self.0.units)
    }
}

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct AmdRegister(&'static AmdRegSpec);

impl AmdRegister {
    pub fn libmc(&self) -> &'static burnerflame::Register {
        &self.0.libmc
    }

    pub fn units(&self) -> &'static [AmdRegUnit] {
        self.0.units
    }

    pub fn overlaps(&self, other: &AmdRegister) -> bool {
        let units_self = self.units();
        let units_other = other.units();
        for unit in units_self {
            if units_other.contains(unit) {
                return true;
            }
        }
        false
    }

    pub fn gpr_registers() -> impl Iterator<Item = AmdRegister> {
        // TODO: More registers
        [rax(), eax(), ecx()].into_iter()
    }
}

impl ILSized for AmdRegister {
    fn il_size(&self) -> ILSize {
        self.0.libmc.il_size()
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

pub fn rax() -> AmdRegister {
    AmdRegister(&RAX)
}

pub fn eax() -> AmdRegister {
    AmdRegister(&EAX)
}

pub fn ecx() -> AmdRegister {
    AmdRegister(&ECX)
}
