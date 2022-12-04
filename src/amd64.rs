#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Reg {
    id: u8,
    mode: OpMode,
}

impl Reg {
    pub fn amd64_codegen(&self) -> u8 {
        self.id
    }

    pub fn has_ex_bit(&self) -> bool {
        match *self {
            RAX => false,
            RCX => false,
            EAX => false,
            _ => panic!("Unknown register"),
        }
    }

    pub fn without_ex_bit(&self) -> Reg {
        Reg {
            id: self.id & 0b111,
            mode: self.mode,
        }
    }
}
pub const RAX: Reg = Reg {
    id: 0,
    mode: OpMode::Bit64,
};
pub const EAX: Reg = Reg {
    id: 0,
    mode: OpMode::Bit32,
};
pub const RCX: Reg = Reg {
    id: 1,
    mode: OpMode::Bit64,
};

#[derive(Debug, Copy, Clone)]
pub struct Imm32 {
    pub int32: i32,
}

#[derive(Debug, Copy, Clone)]
pub enum Immediate {
    Imm32(Imm32),
}

#[derive(Debug)]
pub enum RegImm {
    Reg(Reg),
    Imm(Immediate),
}

#[derive(Debug)]
pub enum RegMem {
    Reg(Reg),
}

mod modrm {
    pub enum FieldMod {
        Direct,
    }

    pub enum FieldReg {
        Reg(super::Reg),
        OpCodeExt(u8),
    }

    impl FieldReg {
        pub fn amd64_codegen(&self) -> u8 {
            match self {
                FieldReg::Reg(reg) => reg.amd64_codegen(),
                FieldReg::OpCodeExt(v) => *v,
            }
        }
    }

    type FieldRM = super::Reg;

    pub struct ModRM {
        pub mode: FieldMod,
        pub reg: FieldReg,
        pub rm: FieldRM,
    }

    impl ModRM {
        pub fn amd64_codegen(&self) -> u8 {
            assert!(!self.rm.has_ex_bit());
            match self.reg {
                FieldReg::OpCodeExt(..) => (),
                FieldReg::Reg(reg) => assert!(reg.has_ex_bit()),
            };

            let mut modrm = match self.mode {
                FieldMod::Direct => 0b11000000,
            };
            modrm |= self.reg.amd64_codegen() << 3;
            modrm |= self.rm.amd64_codegen() << 0;
            modrm
        }
    }
}

struct MovGvIv {
    reg: Reg,
    imm: Imm32,
}

impl MovGvIv {
    pub fn amd64_codegen(&self) -> Vec<u8> {
        assert!(!self.reg.has_ex_bit(), "No support for REX byte");
        let mut base = vec![
            0xC7,
            modrm::ModRM {
                mode: modrm::FieldMod::Direct,
                reg: modrm::FieldReg::OpCodeExt(0),
                rm: self.reg.without_ex_bit(),
            }
            .amd64_codegen(),
        ];
        base.extend_from_slice(&self.imm.int32.to_le_bytes());
        base
    }
}

#[derive(Debug)]
pub struct MovGeneric {
    pub destination: RegMem,
    pub value: RegImm,
}

#[derive(Debug)]
pub enum OpCode {
    Mov(MovGeneric),
    RetNear,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum OpMode {
    Bit64,
    Bit32,
}

impl OpCode {
    pub fn amd64_codegen(&self) -> Vec<u8> {
        match self {
            OpCode::Mov(mov) => match &mov.destination {
                RegMem::Reg(reg) => match &mov.value {
                    RegImm::Imm(imm) => match imm {
                        Immediate::Imm32(imm32) => MovGvIv {
                            reg: *reg,
                            imm: *imm32,
                        }
                        .amd64_codegen(),
                    },
                    RegImm::Reg(..) => todo!(),
                },
            },
            OpCode::RetNear => vec![0xC3],
        }
    }
}
