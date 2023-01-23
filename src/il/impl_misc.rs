use super::*;

impl PlaceholderReg {
    pub fn identifier(&self) -> usize {
        self.identifier
    }

    pub fn size(&self) -> &SizeSpecification {
        &self.size
    }
}

impl MachineReg {
    pub fn unwrap_as_amd64(&self) -> &burnerflame::Register {
        match self {
            Self::AMD64(amd) => amd,
        }
    }

    pub fn unwrap_into_amd64(self) -> burnerflame::Register {
        match self {
            Self::AMD64(amd) => amd,
        }
    }
}

impl SSARegister {
    pub fn is_placeholder(&self) -> bool {
        match self {
            Self::Placeholder(_) => true,
            Self::MachineRegister(_) => false,
        }
    }

    pub fn unwrap_as_machine_register(&self) -> &MachineReg {
        match self {
            Self::Placeholder(_) => panic!("cannot unwrap PlaceholderReg to MachineReg"),
            Self::MachineRegister(x) => x,
        }
    }
}
