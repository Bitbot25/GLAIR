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
            ECX => false,
            _ => panic!("Unknown register"),
        }
    }

    pub fn without_ex_bit(&self) -> Reg {
        Reg {
            id: self.id & 0b111,
            mode: self.mode,
        }
    }

    pub fn ex_bit(&self) -> u8 {
        self.id & 0b1000
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
pub const ECX: Reg = Reg {
    id: 1,
    mode: OpMode::Bit32,
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

mod rex {
    pub struct Rex {
        pub opr_64bit: bool,
        pub modrm_reg_ext: u8,
        pub sib_index_ext: u8,
        pub sib_base_modrm_rm_ext: u8,
    }

    impl Rex {
        pub fn amd64_codegen(&self) -> u8 {
            let mut rex = 0b01000000;
            rex |= (self.opr_64bit as u8) << 3;
            rex |= self.modrm_reg_ext << 2;
            rex |= self.sib_index_ext << 1;
            rex |= self.sib_base_modrm_rm_ext;
            eprintln!("REX: {:08b}", rex);
            rex
        }
    }
}

struct MovGvIv {
    reg: Reg,
    imm: Imm32,
}

impl MovGvIv {
    pub fn amd64_codegen(&self) -> Vec<u8> {
        let mut base = vec![
            0xC7,
            modrm::ModRM {
                mode: modrm::FieldMod::Direct,
                reg: modrm::FieldReg::OpCodeExt(0),
                rm: self.reg.without_ex_bit(),
            }
            .amd64_codegen(),
        ];
        match self.reg.mode {
            OpMode::Bit64 => {
                let rex = rex::Rex {
                    opr_64bit: true,
                    modrm_reg_ext: self.reg.ex_bit(),
                    sib_index_ext: 0,
                    sib_base_modrm_rm_ext: 0,
                }
                .amd64_codegen();
                base.insert(0, rex);
            }
            OpMode::Bit32 => (),
        }
        base.extend_from_slice(&self.imm.int32.to_le_bytes());
        base
    }
}

struct MovGvEv {
    dest: Reg,
    value: Reg,
}

impl MovGvEv {
    pub fn amd64_codegen(&self) -> Vec<u8> {
        todo!()
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
                    RegImm::Reg(val) => MovGvEv {
                        dest: *reg,
                        value: *val,
                    }
                    .amd64_codegen(),
                },
            },
            OpCode::RetNear => vec![0xC3],
        }
    }
}
