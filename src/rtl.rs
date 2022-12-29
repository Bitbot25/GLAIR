use crate::amd64;

#[derive(Clone, Copy)]
pub struct VirtualReg(u32);
#[derive(Clone, Copy)]
pub enum PhysicalReg {
    Amd64(amd64::Reg),
}

#[derive(Clone, Copy)]
pub enum Register {
    Virtual(VirtualReg),
    Phys(PhysicalReg),
}

pub enum Immediate {
    I32(i32),
}

pub enum RValue {
    Immediate(Immediate),
    Register(Register),
}

pub struct Move {
    dest: Register,
    value: RValue,
}

pub enum CompileRtlError {
    VirtualRegister,
    WrongRegisterConversion {
        expected: crate::Arch,
        found: crate::Arch,
    },
}

impl Move {
    fn register_to_amd64_native(reg: Register) -> Result<amd64::Reg, CompileRtlError> {
        let Register::Phys(dest) = reg else {
            return Err(CompileRtlError::VirtualRegister);
        };
        let PhysicalReg::Amd64(dest) = dest; /*else {
                                                 return Err(CompileRtlError::WrongRegisterConversion { expected: crate::Arch::Amd64, found: match dest {
                                                     PhysicalReg::Amd64(_) => unreachable!(),
                                                 } });
                                             };*/

        Ok(dest)
    }

    pub fn compile_amd64(&self) -> Result<Vec<u8>, CompileRtlError> {
        Ok(match &self.value {
            RValue::Immediate(imm) => match imm {
                Immediate::I32(i32) => amd64::MovRegImm32 {
                    reg: Self::register_to_amd64_native(self.dest)?,
                    imm: amd64::Imm32 { int32: *i32 },
                }
                .compile_amd64(),
            },
            RValue::Register(reg) => amd64::MovRegReg {
                dest: Self::register_to_amd64_native(self.dest)?,
                value: Self::register_to_amd64_native(*reg)?,
            }
            .compile_amd64(),
        })
    }
}

pub enum RtlOp {
    Move(Move),
}
