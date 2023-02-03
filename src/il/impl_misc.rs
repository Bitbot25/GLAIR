use super::*;

impl PlaceholderReg {
    pub fn identifier(&self) -> usize {
        self.identifier
    }

    pub fn size(&self) -> &ILSize {
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
    pub fn new(id: usize) -> Self {
        Self {
            id,
            machine_reg: None,
        }
    }

    pub fn machine_reg(id: usize, machine_reg: MachineReg) -> Self {
        Self {
            id,
            machine_reg: Some(machine_reg),
        }
    }

    pub fn has_machine_reg(&self) -> bool {
        self.machine_reg.is_some()
    }

    pub fn unwrap_machine_register(&self) -> &MachineReg {
        self.machine_reg.as_ref().unwrap()
    }

    pub fn unwrap_into_machine_register(self) -> MachineReg {
        self.machine_reg.unwrap()
    }
}

impl Instruction {
    pub fn generate_amd64(&self, assembler: &mut burnerflame::Assembler) {
        match self {
            Instruction::Write(write) => write.generate_amd64(assembler),
            Instruction::DummyUse(_dummy) => (),
            Instruction::Read(_read) => todo!(),
            Instruction::Reserve(reserve) => reserve.generate_amd64(assembler),
            Instruction::Return(ret) => ret.generate_amd64(assembler),
        }
    }

    /// All the variables that are explicitly READ from during this instruction.
    pub fn loaded_variables(&self) -> Vec<&SSARegister> {
        match self {
            Instruction::Write(Write { .. }) => vec![],
            Instruction::DummyUse(DummyUse { register }) => vec![register],
            Instruction::Read(Read {
                target,
                out_data: _,
            }) => vec![target],
            Instruction::Reserve(Reserve { .. }) => vec![],
            Instruction::Return(Return { register }) => match register {
                Some(register) => vec![register],
                None => vec![],
            },
        }
    }
}
