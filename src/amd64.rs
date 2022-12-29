use crate::rtl::ContainsDataType;
use crate::rtl::RegDataType;
use std::fmt;

impl ContainsDataType for Reg {
    fn data_ty(&self) -> RegDataType {
        match self.mode {
            OpMode::Bit64 => RegDataType::Int64,
            OpMode::Bit32 => RegDataType::Int32,
            OpMode::Bit16 => RegDataType::Int16,
            OpMode::Bit8 => RegDataType::Int8,
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Reg {
    id: u8,
    mode: OpMode,
}

impl Reg {
    pub fn compile_amd64(&self) -> u8 {
        self.id
    }

    pub fn has_ex_bit(&self) -> bool {
        self.ex_bit() == 1
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
pub const AX: Reg = Reg {
    id: 0,
    mode: OpMode::Bit16,
};
pub const A: Reg = Reg {
    id: 0,
    mode: OpMode::Bit8,
};
pub const RCX: Reg = Reg {
    id: 1,
    mode: OpMode::Bit64,
};
pub const ECX: Reg = Reg {
    id: 1,
    mode: OpMode::Bit32,
};
pub const CX: Reg = Reg {
    id: 1,
    mode: OpMode::Bit16,
};
pub const C: Reg = Reg {
    id: 1,
    mode: OpMode::Bit8,
};

#[derive(Copy, Clone)]
pub union Imm32 {
    pub int32: i32,
    pub uint32: u32,
}

impl fmt::Debug for Imm32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Imm32")
            .field("int32", unsafe { &self.int32 })
            .field("uint32", unsafe { &self.uint32 })
            .finish()
    }
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
        pub fn compile_amd64(&self) -> u8 {
            match self {
                FieldReg::Reg(reg) => reg.compile_amd64(),
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
        pub fn compile_amd64(&self) -> u8 {
            assert!(!self.rm.has_ex_bit());
            match self.reg {
                FieldReg::OpCodeExt(..) => (),
                FieldReg::Reg(reg) => assert!(!reg.has_ex_bit()),
            };

            let mut modrm = match self.mode {
                FieldMod::Direct => 0b11000000,
            };
            modrm |= self.reg.compile_amd64() << 3;
            modrm |= self.rm.compile_amd64() << 0;
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
        pub fn compile_amd64(&self) -> u8 {
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

#[derive(Debug)]
pub struct MovRegImm32 {
    pub reg: Reg,
    pub imm: Imm32,
}

impl MovRegImm32 {
    pub fn compile_amd64(&self) -> Vec<u8> {
        let mut base = vec![
            0xC7,
            modrm::ModRM {
                mode: modrm::FieldMod::Direct,
                reg: modrm::FieldReg::OpCodeExt(0),
                rm: self.reg.without_ex_bit(),
            }
            .compile_amd64(),
        ];
        match self.reg.mode {
            OpMode::Bit64 => {
                let rex = rex::Rex {
                    opr_64bit: true,
                    modrm_reg_ext: self.reg.ex_bit(),
                    sib_index_ext: 0,
                    sib_base_modrm_rm_ext: 0,
                }
                .compile_amd64();
                base.insert(0, rex);
            }
            OpMode::Bit32 | OpMode::Bit16 | OpMode::Bit8 => (),
        }
        base.extend_from_slice(unsafe { &self.imm.int32.to_le_bytes() });
        base
    }
}

#[derive(Debug)]
pub struct MovRegReg {
    pub dest: Reg,
    pub value: Reg,
}

impl MovRegReg {
    pub fn compile_amd64(&self) -> Vec<u8> {
        let dest_64bit = self.dest.mode == OpMode::Bit64;
        let value_64bit = self.value.mode == OpMode::Bit64;
        let mut buf = if dest_64bit || value_64bit {
            let rex = rex::Rex {
                opr_64bit: true,
                modrm_reg_ext: self.value.ex_bit(),
                sib_index_ext: 0,
                sib_base_modrm_rm_ext: self.dest.ex_bit(),
            }
            .compile_amd64();
            vec![rex]
        } else {
            Vec::new()
        };
        buf.push(0x89);
        let modrm = modrm::ModRM {
            mode: modrm::FieldMod::Direct,
            reg: modrm::FieldReg::Reg(self.value.without_ex_bit()),
            rm: self.dest.without_ex_bit(),
        }
        .compile_amd64();
        buf.push(modrm);
        buf
    }
}

#[derive(Debug)]
pub struct RetNear;

impl RetNear {
    pub fn compile_amd64(&self) -> u8 {
        0xc3
    }
}

#[derive(Debug)]
pub enum OpCode {
    MovRegImm32(MovRegImm32),
    MovRegReg(MovRegReg),
    RetNear(RetNear),
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum OpMode {
    Bit64,
    Bit32,
    Bit16,
    Bit8,
}

impl OpCode {
    pub fn compile_amd64(&self) -> Vec<u8> {
        match self {
            OpCode::MovRegImm32(mov) => mov.compile_amd64(),
            OpCode::MovRegReg(mov) => mov.compile_amd64(),
            OpCode::RetNear(r) => vec![r.compile_amd64()],
        }
    }
}
