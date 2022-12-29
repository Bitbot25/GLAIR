use crate::amd64;
use crate::cfg;

#[derive(Clone, Copy)]
pub enum RegDataType {
    Int8,
    Int16,
    Int32,
    Int64,
    Custom(usize),
}

#[derive(Clone, Copy)]
pub struct VirtualReg {
    id: u32,
    data_ty: RegDataType,
}

pub trait ContainsDataType {
    fn data_ty(&self) -> RegDataType;
}

#[derive(Clone, Copy)]
pub struct StackRegister {
    scope_slot: usize,
    data_ty: RegDataType,
}

#[derive(Clone, Copy)]
pub enum PhysicalReg {
    Amd64(amd64::Reg),
    Stack(StackRegister),
}

impl ContainsDataType for PhysicalReg {
    fn data_ty(&self) -> RegDataType {
        match self {
            PhysicalReg::Amd64(r) => r.data_ty(),
            PhysicalReg::Stack(stack) => stack.data_ty,
        }
    }
}

#[derive(Clone, Copy)]
pub enum Register {
    Virtual(VirtualReg),
    Phys(PhysicalReg),
}

impl ContainsDataType for Register {
    fn data_ty(&self) -> RegDataType {
        match self {
            Register::Virtual(vir) => vir.data_ty,
            Register::Phys(phys) => phys.data_ty(),
        }
    }
}

pub enum Immediate {
    I32(i32),
}

pub enum RValue {
    Immediate(Immediate),
    Register(Register),
}

pub struct Move {
    pub dest: Register,
    pub value: RValue,
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
        match dest {
            PhysicalReg::Stack(_stack_reg) => todo!(),
            PhysicalReg::Amd64(dest) => Ok(dest),
        }
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

#[derive(Debug, PartialEq, Eq)]
pub enum CallingConvention {
    C,
}

pub struct Return {
    pub value: Option<Register>,
    pub cc: CallingConvention,
}

fn validate_alignment_amd64(size: usize) {
    if size % 16 != 0 || size == 0 {
        panic!("Invalid stack alignment");
    }
}

impl Return {
    pub fn compile_amd64(&self) -> Vec<u8> {
        assert_eq!(self.cc, CallingConvention::C);
        match self.value {
            Some(Register::Phys(PhysicalReg::Amd64(ret_v))) => {
                let mut buf = Vec::new();
                let ret_t = ret_v.data_ty();
                match ret_t {
                    RegDataType::Int8 | RegDataType::Int16 => todo!(),
                    RegDataType::Int32 => {
                        // Place into eax
                        buf.append(
                            &mut amd64::MovRegReg {
                                dest: amd64::EAX,
                                value: ret_v,
                            }
                            .compile_amd64(),
                        );
                    }
                    RegDataType::Int64 => {
                        // Place into rax
                        buf.append(
                            &mut amd64::MovRegReg {
                                dest: amd64::RAX,
                                value: ret_v,
                            }
                            .compile_amd64(),
                        );
                    }
                    RegDataType::Custom(custom_sz) => {
                        if custom_sz <= 8 {
                            let bits = custom_sz * 8;
                            let reg = if bits > 8 {
                                if bits > 16 {
                                    if bits > 32 {
                                        amd64::RAX
                                    } else {
                                        amd64::EAX
                                    }
                                } else {
                                    amd64::AX
                                }
                            } else {
                                amd64::A
                            };

                            buf.append(
                                &mut amd64::MovRegReg {
                                    dest: reg,
                                    value: ret_v,
                                }
                                .compile_amd64(),
                            );
                        } else {
                            validate_alignment_amd64(custom_sz);
                            panic!("no support for stack")
                        }
                    }
                };
                buf.push(amd64::RetNear.compile_amd64());
                buf
            }
            Some(Register::Phys(PhysicalReg::Stack(_stack_r))) => {
                panic!("no support for stack registers")
            }
            // TODO: vvvvvv
            Some(Register::Virtual(_vir)) => {
                panic!("cannot return virtual register. (TODO: Return error type instead)")
            }
            None => vec![amd64::RetNear.compile_amd64()], // FIXME: Clear return registers if they are "occupied". We need a data structure to keep track of this.
        }
    }
}

pub struct Call<'cfg> {
    callee: Function<'cfg>,
}

impl<'cfg> Call<'cfg> {
    pub fn compile_amd64(&self) -> Vec<u8> {
        todo!()
    }
}

pub struct Function<'cfg> {
    block: cfg::Block<'cfg, RtlOp<'cfg>>,
}

pub enum RtlOp<'cfg> {
    Move(Move),
    Return(Return),
    Call(Call<'cfg>),
}

impl<'cfg> RtlOp<'cfg> {
    pub fn compile_amd64(&self) -> Result<Vec<u8>, CompileRtlError> {
        match self {
            RtlOp::Move(mov) => mov.compile_amd64(),
            RtlOp::Return(ret) => Ok(ret.compile_amd64()),
            RtlOp::Call(call) => Ok(call.compile_amd64()),
        }
    }
}
