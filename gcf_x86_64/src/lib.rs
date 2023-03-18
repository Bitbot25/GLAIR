#![allow(non_camel_case_types)]
use std::{
    io::{self, Write},
    ops::BitOr,
};

use enum_map::{enum_map, Enum, EnumMap};
use lazy_static::lazy_static;

pub enum Unit {
    al,
    ah,

    cl,
    ch,

    dl,
    dh,

    bl,
    bh,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct EncodingFlags {
    bits: u8,
}

impl EncodingFlags {
    pub const FLAG_EXCEPTIONAL_REX: EncodingFlags = EncodingFlags { bits: 0b00000001 };

    #[inline]
    pub const fn none() -> Self {
        EncodingFlags { bits: 0 }
    }

    #[inline]
    pub const fn max() -> Self {
        EncodingFlags { bits: u8::MAX }
    }

    #[inline]
    pub const fn flag_count(&self) -> usize {
        self.bits.count_ones() as usize
    }

    #[inline]
    pub const fn from_bits(bits: u8) -> Self {
        EncodingFlags { bits }
    }

    #[inline]
    pub const fn has(self, other: EncodingFlags) -> bool {
        (self.bits & other.bits) == other.bits
    }
}

impl BitOr for EncodingFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        EncodingFlags {
            bits: self.bits | rhs.bits,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Encoding {
    bits: u8,
}

impl Encoding {
    const BITS_MAIN: usize = 3;
    const BITS_EX: usize = 1;

    const AVAILABLE_BITS: usize = 8 - Self::BITS_MAIN - Self::BITS_EX;

    const MAIN_MAX: usize = (1 << Self::BITS_MAIN) - 1;
    const FLAGS_MAX: usize = (1 << Self::AVAILABLE_BITS) - 1;
    const EX_MAX: usize = (1 << Self::BITS_EX) - 1;

    const EX_MASK: u8 = (Self::EX_MAX << Self::BITS_MAIN) as u8;
    const FLAGS_MASK: u8 = ((Self::FLAGS_MAX << Self::BITS_MAIN) << Self::BITS_EX) as u8;
    const MAIN_MASK: u8 = (Self::MAIN_MAX) as u8;

    #[inline]
    pub const fn new(enc: u8, ex: u8, flags: EncodingFlags) -> Self {
        debug_assert!(enc as usize <= Self::MAIN_MAX);
        debug_assert!(ex as usize <= Self::EX_MAX);
        debug_assert!(flags.bits as usize <= Self::FLAGS_MAX);

        Self {
            bits: (enc
                | (ex << Self::BITS_MAIN)
                | ((flags.bits << Self::BITS_MAIN) << Self::BITS_EX)) as u8,
        }
    }

    #[inline]
    pub const fn main(self) -> u8 {
        self.bits & Self::MAIN_MASK
    }

    #[inline]
    pub const fn ex(self) -> u8 {
        (self.bits & Self::EX_MASK) >> Self::BITS_MAIN
    }

    #[inline]
    pub const fn flags(self) -> EncodingFlags {
        EncodingFlags::from_bits((self.bits & Self::FLAGS_MASK) >> Self::BITS_MAIN >> Self::BITS_EX)
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone, Enum)]
pub enum Register {
    al,
    ah,

    cl,
    ch,

    dl,
    dh,

    bl,
    bh,
}

lazy_static! {
    static ref REGISTER_ENCODING: EnumMap<Register, Encoding> = enum_map! {
        Register::al => Encoding::new(0, 0, EncodingFlags::none()),
        Register::ah => Encoding::new(4, 0, EncodingFlags::none()),

        Register::cl => Encoding::new(1, 0, EncodingFlags::none()),
        Register::ch => Encoding::new(5, 0, EncodingFlags::none()),

        Register::dl => Encoding::new(2, 0, EncodingFlags::none()),
        Register::dh => Encoding::new(6, 0, EncodingFlags::none()),

        Register::bl => Encoding::new(3, 0, EncodingFlags::none()),
        Register::bh => Encoding::new(7, 0, EncodingFlags::none()),
    };
}

impl Register {
    #[inline]
    pub fn encoding(self) -> Encoding {
        REGISTER_ENCODING[self]
    }
}

#[repr(u8)]
#[rustfmt::skip]
pub enum AddressingMode {
    Direct          = 0b11,
    IndirectGeneric = 0b00,
    IndirectDisp8   = 0b01,
    IndirectDisp32  = 0b10,
}

#[derive(Debug)]
pub struct ModRM {
    /// u2
    pub addrmode: u8,
    /// G operand: u3
    pub reg: u8,
    /// E operand: u3
    pub rm: u8,
}

impl EncodeInto for ModRM {
    fn encode_into<W: Write>(&self, buf: &mut W) -> io::Result<()> {
        let modrm = (self.addrmode << 6) | (self.reg << 3) | self.rm;
        buf.write_all(&[modrm])
    }
}

#[derive(Debug)]
pub struct Rex {
    /// W operand: u1
    pub operand_64bit: bool,
    /// R operand - Extension of G operand: u1
    pub modrm_reg_extension: u8,
    /// X operand: u1
    pub sib_index_extension: u8,
    /// B operand - Extension of E operand or SIB base: u1
    pub modrm_rm_sib_base_extension: u8,
}

impl EncodeInto for Rex {
    fn encode_into<W: Write>(&self, buf: &mut W) -> io::Result<()> {
        let rex = 0b01000000
            | ((self.operand_64bit as u8) << 3)
            | (self.modrm_reg_extension << 2)
            | (self.sib_index_extension << 1)
            | self.modrm_rm_sib_base_extension;
        buf.write_all(&[rex])
    }
}

#[derive(Debug)]
pub struct Sib {
    /// u2
    pub scale_mode: u8,
    /// u3
    pub index: u8,
    /// u3
    pub base: u8,
}

impl EncodeInto for Sib {
    fn encode_into<W: Write>(&self, buf: &mut W) -> io::Result<()> {
        let sib = self.scale_mode | (self.index << 6) | self.base;
        buf.write_all(&[sib])
    }
}

#[derive(Debug)]
pub enum Displacement {
    Int8(u8),
    Int16(u16),
    Int32(u32),
}

impl EncodeInto for Displacement {
    fn encode_into<W: Write>(&self, buf: &mut W) -> io::Result<()> {
        match self {
            Displacement::Int8(u8) => buf.write_all(&[*u8]),
            Displacement::Int16(u16) => buf.write_all(&u16.to_le_bytes()),
            Displacement::Int32(u32) => buf.write_all(&u32.to_le_bytes()),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Immediate {
    Int8(u8),
    Int16(u16),
    Int32(u32),
    Int64(u64),
}

impl Immediate {
    pub fn bitmode(&self) -> BitMode {
        match self {
            Immediate::Int8(_) => BitMode::Bit8,
            Immediate::Int16(_) => BitMode::Bit16,
            Immediate::Int32(_) => BitMode::Bit32,
            Immediate::Int64(_) => BitMode::Bit64,
        }
    }
}

impl EncodeInto for Immediate {
    fn encode_into<W: Write>(&self, buf: &mut W) -> io::Result<()> {
        match self {
            Immediate::Int8(u8) => buf.write_all(&[*u8]),
            Immediate::Int16(u16) => buf.write_all(&u16.to_le_bytes()),
            Immediate::Int32(u32) => buf.write_all(&u32.to_le_bytes()),
            Immediate::Int64(u64) => buf.write_all(&u64.to_le_bytes()),
        }
    }
}

#[derive(Clone, Copy, Debug, Enum)]
#[repr(u8)]
pub enum OpCode {
    mov_r8m8_imm8,
    mov_r16m16_imm16,
    mov_r32m32_imm32,
    mov_r64m64_simm32,

    mov_r64_imm64(Register),
    mov_r32_imm32(Register),
    mov_r16_imm16(Register),
    mov_r8_imm8(Register),

    mov_r64m64_r64,
    mov_r32m32_r32,
    mov_r16m16_r16,
    mov_r8m8_r8,

    retn,
    retn_imm16,

    nop,
}

impl OpCode {
    pub fn encoding(&self) -> (u8, u8) {
        match self {
            Self::mov_r8m8_imm8 => (0xc6, 0),
            Self::mov_r16m16_imm16 => (0xc7, 0),
            Self::mov_r32m32_imm32 => (0xc7, 0),
            Self::mov_r64m64_simm32 => (0xc7, 0),

            Self::mov_r64_imm64(r) | Self::mov_r32_imm32(r) | Self::mov_r16_imm16(r) => {
                (0xb8 + r.encoding().main(), 0)
            }
            Self::mov_r8_imm8(r) => (0xb0 + r.encoding().main(), 0),

            Self::mov_r64m64_r64 | Self::mov_r32m32_r32 | Self::mov_r16m16_r16 => (0x89, 0),
            Self::mov_r8m8_r8 => (0x88, 0),

            Self::retn => (0xc3, 0),
            Self::retn_imm16 => (0xc2, 0),

            Self::nop => (0x90, 0),
        }
    }
}

#[derive(Debug)]
pub struct Instruction {
    pub legacy_66h: bool,
    pub modrm: Option<ModRM>,
    pub rex: Option<Rex>,
    pub sib: Option<Sib>,
    pub displacement: Option<Displacement>,
    pub opcode: u8,
    pub immediate: Option<Immediate>,
}

pub trait EncodeInto {
    fn encode_into<W: Write>(&self, buf: &mut W) -> io::Result<()>;
}

impl EncodeInto for Instruction {
    fn encode_into<W: Write>(&self, buf: &mut W) -> io::Result<()> {
        if let Some(rex) = &self.rex {
            rex.encode_into(buf)?;
        }

        buf.write_all(&[self.opcode])?;

        if let Some(modrm) = &self.modrm {
            modrm.encode_into(buf)?;
        }

        if let Some(sib) = &self.sib {
            sib.encode_into(buf)?;
        }

        if let Some(displacement) = &self.displacement {
            displacement.encode_into(buf)?;
        }

        if let Some(immediate) = &self.immediate {
            immediate.encode_into(buf)?;
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BitMode {
    Bit64,
    Bit32,
    Bit16,
    Bit8,
    NA,
}

#[derive(Copy, Clone)]
pub struct OpFlags {
    bits: u8,
}

#[rustfmt::skip]
impl OpFlags {
    pub const MODRM: OpFlags = OpFlags { bits: 0b00000001 };
    pub const REX: OpFlags   = OpFlags { bits: 0b00000010 };
}

impl OpFlags {
    #[inline]
    pub fn has(self, other: OpFlags) -> bool {
        (self.bits & other.bits) == other.bits
    }
}

#[inline(always)]
fn use_rex_p(mode: BitMode, registers: &[Register]) -> bool {
    mode == BitMode::Bit64
        || registers.iter().any(|r| {
            r.encoding()
                .flags()
                .has(EncodingFlags::FLAG_EXCEPTIONAL_REX)
                || r.encoding().ex() != 0
        })
}

pub fn exists_override_prefix(mode: BitMode) -> bool {
    match mode {
        BitMode::Bit64 => true,
        BitMode::Bit32 => false,
        BitMode::Bit16 => true,
        BitMode::Bit8 => false,
        BitMode::NA => false,
    }
}

pub fn mov_reg_reg(mode: BitMode, to: Register, from: Register) -> Instruction {
    Instruction {
        legacy_66h: mode == BitMode::Bit16,
        modrm: Some(ModRM {
            addrmode: AddressingMode::Direct as u8,
            reg: to.encoding().main(),
            rm: from.encoding().main(),
        }),
        rex: if use_rex_p(mode, &[to, from]) {
            Some(Rex {
                operand_64bit: mode == BitMode::Bit64,
                modrm_reg_extension: to.encoding().ex(),
                sib_index_extension: 0,
                modrm_rm_sib_base_extension: from.encoding().ex(),
            })
        } else {
            None
        },
        sib: None,
        displacement: None,
        opcode: match mode {
            BitMode::Bit64 => OpCode::mov_r64m64_r64,
            BitMode::Bit32 => OpCode::mov_r32m32_r32,
            BitMode::Bit16 => OpCode::mov_r16m16_r16,
            BitMode::Bit8 => OpCode::mov_r8m8_r8,
            BitMode::NA => panic!("invalid bit mode for mov instruction"),
        }
        .encoding()
        .0,
        immediate: None,
    }
}

pub fn mov_reg_imm(mode: BitMode, to: Register, from: Immediate) -> Instruction {
    assert_eq!(mode, from.bitmode());
    Instruction {
        legacy_66h: mode == BitMode::Bit16,
        modrm: None,
        rex: if use_rex_p(mode, &[to]) {
            Some(Rex {
                operand_64bit: mode == BitMode::Bit64,
                modrm_reg_extension: 0,
                sib_index_extension: 0,
                modrm_rm_sib_base_extension: to.encoding().ex(),
            })
        } else {
            None
        },
        sib: None,
        displacement: None,
        opcode: match mode {
            BitMode::Bit8 => OpCode::mov_r8_imm8(to),
            BitMode::Bit16 => OpCode::mov_r16_imm16(to),
            BitMode::Bit32 => OpCode::mov_r32_imm32(to),
            BitMode::Bit64 => OpCode::mov_r64_imm64(to),
            BitMode::NA => panic!("invalid bitmode for mov instruction"),
        }
        .encoding()
        .0,
        immediate: Some(from),
    }
}

pub fn retn(mode: BitMode) -> Instruction {
    // if the mode is not 64-bit; check that there is an override prefix. BitMode::Bit64 has an override prefix, so we can remove the !=
    // assert!(mode != BitMode::Bit64 || exists_override_prefix(mode));
    assert!(exists_override_prefix(mode));
    Instruction {
        legacy_66h: mode == BitMode::Bit16,
        modrm: None,
        rex: None,
        sib: None,
        displacement: None,
        opcode: OpCode::retn.encoding().0,
        immediate: None,
    }
}

pub fn nop() -> Instruction {
    Instruction { legacy_66h: false, modrm: None, rex: None, sib: None, displacement: None, opcode: OpCode::nop.encoding().0, immediate: None }
}

#[cfg(test)]
mod tests {
    use crate::{Encoding, EncodingFlags};

    #[test]
    fn encoding_pack() {
        fn test_encoding(main: u8, ex: u8, flags: EncodingFlags) {
            let encoding = Encoding::new(main, ex, flags);
            assert_eq!(encoding.main(), main);
            assert_eq!(encoding.ex(), ex);
            assert_eq!(encoding.flags(), flags);
        }

        test_encoding(1, 0, EncodingFlags::FLAG_EXCEPTIONAL_REX);
        test_encoding(1, 1, EncodingFlags::FLAG_EXCEPTIONAL_REX);
        test_encoding(0, 0, EncodingFlags::FLAG_EXCEPTIONAL_REX);
        test_encoding(0, 0, EncodingFlags::none());
    }
}
