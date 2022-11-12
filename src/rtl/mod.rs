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
pub struct VirRegister {
    pub bytes: usize,
    pub n: usize,
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum Register {
    Vir(VirRegister),
    Real(RealRegister),
}

impl Register {
    #[inline]
    pub fn unwrap_real(&self) -> &RealRegister {
        match self {
            Register::Vir(pseudo) => panic!("Pseudo register {} was not resolved.", pseudo.n),
            Register::Real(real) => real,
        }
    }

    pub fn sz(&self) -> usize {
        match self {
            Register::Real(real) => real.sz(),
            Register::Vir(pseudo) => pseudo.bytes,
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

pub struct OpAdd {
    pub to: Register,
    pub val: RValue,
}

pub struct OpSub {
    pub from: Register,
    pub val: RValue,
}

pub struct OpMul {
    pub val: Register,
    pub with: RValue,
}

pub struct OpDiv {
    pub val: Register,
    pub with: RValue,
}

pub enum Op {
    Copy(OpCopy),
    Add(OpAdd),
    Sub(OpSub),
    Mul(OpMul),
    Div(OpDiv),
}

pub type Ops = Vec<Op>;

pub struct Block {
    pub name: Option<String>,
    pub ops: Ops,
    pub metadata: (),
}

fn promote_register(reg: &mut Register, mut promote: impl FnMut(&VirRegister) -> RealRegister) {
    match reg {
        Register::Vir(vir) => *reg = Register::Real(promote(vir)),
        Register::Real(..) => (),
    }
}

fn promote_rvalue(rvalue: &mut RValue, promote: impl FnMut(&VirRegister) -> RealRegister) {
    match rvalue {
        RValue::Lit(..) => (),
        RValue::Register(reg) => promote_register(reg, promote),
    }
}

pub fn promote_registers_in_op(op: &mut Op, mut promote: impl FnMut(&VirRegister) -> RealRegister) {
    match op {
        Op::Copy(OpCopy { to, from }) => {
            promote_register(to, &mut promote);
            promote_rvalue(from, &mut promote);
        }
        Op::Add(OpAdd { to, val }) => {
            promote_register(to, &mut promote);
            promote_rvalue(val, &mut promote);
        }
        Op::Sub(OpSub { from, val }) => {
            promote_register(from, &mut promote);
            promote_rvalue(val, &mut promote);
        }
        Op::Mul(OpMul { val, with }) => {
            promote_register(val, &mut promote);
            promote_rvalue(with, &mut promote);
        }
        Op::Div(OpDiv { val, with }) => {
            promote_register(val, &mut promote);
            promote_rvalue(with, &mut promote);
        }
    }
}

pub fn promote_registers_in_ops(
    ops: &mut Ops,
    mut promote: impl FnMut(&VirRegister) -> RealRegister,
) {
    for op in ops {
        promote_registers_in_op(op, &mut promote);
    }
}
