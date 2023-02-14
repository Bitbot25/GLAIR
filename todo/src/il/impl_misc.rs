use super::*;

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
