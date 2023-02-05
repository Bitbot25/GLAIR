use super::*;
use burnerflame::{AssmMov, AssmRet};

impl Reserve {
    pub fn generate_amd64(&self, _assm: &mut burnerflame::Assembler) {
        todo!()
    }
}

impl Return {
    pub fn generate_amd64(&self, assm: &mut burnerflame::Assembler) {
        match self.register {
            Some(_) => todo!(),
            None => assm.ret(),
        }
    }
}

impl Write {
    pub fn generate_amd64(&self, buf: &mut burnerflame::Assembler) {
        let Self { destination, value } = self;
        let destination = destination.unwrap_machine_register().as_raw_amd64();
        let destination_sz = destination.il_size().width();

        match destination_sz {
            8 => match value {
                RValue::Immediate(Immediate::U32(_imm_u32)) => {
                    panic!("Cannot move 32-bit immediate into 64-bit register.");
                    // buf.mov(burnerflame::Register64::new(destination), *imm_u32)
                }
                RValue::SSARegister(value_reg) => {
                    let value_reg = value_reg.unwrap_machine_register().as_raw_amd64();
                    match value_reg.il_size().width() {
                        8 => buf.mov(
                            burnerflame::Register64::new(*destination),
                            burnerflame::Register64::new(*value_reg),
                        ),
                        n => panic!("Cannot move {n}-byte register into 8-byte register."),
                    }
                }
            },
            4 => match value {
                RValue::Immediate(Immediate::U32(imm_u32)) => {
                    buf.mov(burnerflame::Register32::new(*destination), *imm_u32)
                }
                RValue::SSARegister(value_reg) => {
                    let value_reg = value_reg.unwrap_machine_register().as_raw_amd64();
                    match value_reg.il_size().width() {
                        4 => buf.mov(
                            burnerflame::Register32::new(*destination),
                            burnerflame::Register32::new(*value_reg),
                        ),
                        n => panic!("Cannot move {n}-byte register into 4-byte register"),
                    }
                }
            },
            2 => match value {
                RValue::Immediate(Immediate::U32(_imm_u32)) => {
                    panic!("Cannot move 32-bit immediate into 16-bit register.");
                    // buf.mov(burnerflame::Register16::new(destination), imm_u32)
                }
                RValue::SSARegister(value_reg) => {
                    let value_reg = value_reg.unwrap_machine_register().as_raw_amd64();
                    match value_reg.il_size().width() {
                        2 => buf.mov(
                            burnerflame::Register16::new(*destination),
                            burnerflame::Register16::new(*value_reg),
                        ),
                        n => panic!("Cannot move {n}-byte register into 4-byte register"),
                    }
                }
            },
            _ => todo!(),
        }
    }
}
