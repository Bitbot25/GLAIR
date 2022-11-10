pub mod amd64;
pub mod debug;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum RealRegister {
    Amd64(amd64::Amd64Register),
}

impl RealRegister {
    pub fn sz(&self) -> usize {
        match self {
            RealRegister::Amd64(reg) => reg.sz(),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct PseudoRegister {
    pub bytes: usize,
    pub n: usize,
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum Register {
    Pseudo(PseudoRegister),
    Real(RealRegister),
}

impl Register {
    #[inline]
    pub fn unwrap_real(&self) -> &RealRegister {
        match self {
            Register::Pseudo(pseudo) => panic!("Pseudo register {} was not resolved.", pseudo.n),
            Register::Real(real) => real,
        }
    }

    pub fn sz(&self) -> usize {
        match self {
            Register::Real(real) => real.sz(),
            Register::Pseudo(pseudo) => pseudo.bytes,
        }
    }
}

pub enum Lit {
    LitU8(u8),
    LitU32(u32),
}

impl Lit {
    pub fn sz(&self) -> usize {
        match self {
            Lit::LitU8(..) => 1,
            Lit::LitU32(..) => 4,
        }
    }
}

pub enum RValue {
    Register(Register),
    Lit(Lit),
}

impl RValue {
    pub fn sz(&self) -> usize {
        match self {
            RValue::Register(reg) => reg.sz(),
            RValue::Lit(lit) => lit.sz(),
        }
    }
}

pub struct OpCopy {
    pub to: Register,
    pub from: RValue,
}

pub struct OpSub {
    pub from: Register,
    pub val: RValue,
}

pub enum Op {
    Copy(OpCopy),
    Sub(OpSub),
}

pub type Ops = Vec<Op>;

pub struct Block {
    pub name: Option<String>,
    pub ops: Ops,
    pub metadata: (),
}

fn promote_register(reg: &mut Register, mut promote: impl FnMut(PseudoRegister) -> RealRegister) {
    match reg {
        Register::Pseudo(pseudo) => *reg = Register::Real(promote(*pseudo)),
        Register::Real(..) => (),
    }
}

fn promote_rvalue(rvalue: &mut RValue, promote: impl FnMut(PseudoRegister) -> RealRegister) {
    match rvalue {
        RValue::Lit(..) => (),
        RValue::Register(reg) => promote_register(reg, promote),
    }
}

pub fn promote_registers_in_op(
    op: &mut Op,
    mut promote: impl FnMut(PseudoRegister) -> RealRegister,
) {
    match op {
        Op::Copy(OpCopy { to, from }) => {
            promote_register(to, &mut promote);
            promote_rvalue(from, &mut promote);
        }
        Op::Sub(OpSub { from, val }) => {
            promote_register(from, &mut promote);
            promote_rvalue(val, &mut promote);
        }
    }
}

pub fn promote_registers_in_ops(
    ops: &mut Ops,
    mut promote: impl FnMut(PseudoRegister) -> RealRegister,
) {
    for op in ops {
        promote_registers_in_op(op, &mut promote);
    }
}
