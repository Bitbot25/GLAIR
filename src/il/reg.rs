use super::{amd::AmdRegister, ILSize, ILSized};
use std::hash;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MachineReg {
    AMD64(AmdRegister),
}

impl ILSized for MachineReg {
    fn il_size(&self) -> ILSize {
        match self {
            MachineReg::AMD64(r) => r.il_size(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct SSARegister {
    id: usize,
    size: ILSize,
    machine_reg: Option<MachineReg>,
}

impl SSARegister {
    #[inline]
    pub fn new(id: usize, size: ILSize) -> Self {
        Self {
            id,
            size,
            machine_reg: None,
        }
    }

    #[inline]
    pub fn of_mc_register(id: usize, mc_register: MachineReg) -> Self {
        Self {
            id,
            size: mc_register.il_size(),
            machine_reg: Some(mc_register),
        }
    }

    #[inline]
    pub fn id(&self) -> usize {
        self.id
    }

    #[inline]
    pub fn size(&self) -> ILSize {
        self.size
    }

    #[inline]
    pub fn mc_register(&self) -> Option<&MachineReg> {
        self.machine_reg.as_ref()
    }
}

impl ILSized for SSARegister {
    fn il_size(&self) -> ILSize {
        self.size
    }
}

impl PartialEq for SSARegister {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for SSARegister {}

impl hash::Hash for SSARegister {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.id);
    }
}

impl MachineReg {
    pub fn as_raw_amd64(&self) -> &burnerflame::Register {
        match self {
            Self::AMD64(reg) => reg.libmc(),
        }
    }
}
