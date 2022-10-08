pub type Reg = usize;

pub const REG_X86_EAX: Reg = 0;
pub const REG_X86_ECX: Reg = 1;
pub const REG_X86_EDX: Reg = 2;
pub const REG_X86_ESP: Reg = 3;
pub const REG_AMD64_RSP: Reg = 4;

impl Codegen for Reg {
    fn nasm(&self) -> String {
        match *self {
            REG_X86_EAX => "eax".to_string(),
            REG_X86_ECX => "ecx".to_string(),
            REG_X86_EDX => "edx".to_string(),
            REG_X86_ESP => "esp".to_string(),
            REG_AMD64_RSP => "rsp".to_string(),
            _ => unimplemented!(),
        }
    }
}

impl AsWordTy for Reg {
    fn word_ty(&self) -> WordTy {
        match *self {
            REG_X86_EAX | REG_X86_ECX | REG_X86_EDX | REG_X86_ESP => WordTy::DWord,
            REG_AMD64_RSP => WordTy::QWord,
            _ => unimplemented!(),
        }
    }
}

pub trait Codegen {
    fn nasm(&self) -> String;
}

pub trait AsWordTy {
    fn word_ty(&self) -> WordTy;
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum WordTy {
    DWord, // u32
    QWord, // u64
}

impl Codegen for WordTy {
    fn nasm(&self) -> String {
        match self {
            WordTy::DWord => "dword".to_string(),
            WordTy::QWord => "qword".to_string(),
        }
    }
}

#[derive(Debug)]
pub enum DerefPlace {
    Reg(WordTy, Reg),
    Addr(WordTy, isize),
    Sub(Box<(DerefPlace, DerefPlace)>),
}

impl Codegen for DerefPlace {
    fn nasm(&self) -> String {
        match self {
            DerefPlace::Reg(_wty, reg) => reg.nasm(),
            DerefPlace::Addr(_wty, addr) => addr.to_string(),
            DerefPlace::Sub(operands) => format!("{} - {}", operands.0.nasm(), operands.1.nasm()),
        }
    }
}

impl AsWordTy for DerefPlace {
    fn word_ty(&self) -> WordTy {
        match self {
            DerefPlace::Reg(wty, ..) => *wty,
            DerefPlace::Addr(wty, ..) => *wty,
            DerefPlace::Sub(operands) => {
                assert_eq!(operands.0.word_ty(), operands.1.word_ty());
                operands.0.word_ty()
            }
        }
    }
}

#[derive(Debug)]
pub enum Place {
    Reg(Reg),
    Deref(DerefPlace),
}

impl Codegen for Place {
    fn nasm(&self) -> String {
        match self {
            Place::Reg(reg) => reg.nasm(),
            Place::Deref(deref) => format!("{} [{}]", deref.word_ty().nasm(), deref.nasm()),
        }
    }
}

#[derive(Debug)]
pub enum Value {
    I32(i32),
    U32(u32),
    Place(Place),
}

impl Codegen for Value {
    fn nasm(&self) -> String {
        match self {
            Value::I32(val) => val.to_string(),
            Value::U32(val) => val.to_string(),
            Value::Place(place) => place.nasm(),
        }
    }
}

#[derive(Debug)]
pub enum Op {
    Move(Place, Value),
    Sub(Place, Value),
    Add(Place, Value),
}

impl Codegen for Op {
    fn nasm(&self) -> String {
        match self {
            Op::Move(place, value) => format!("mov {}, {}", place.nasm(), value.nasm()),
            Op::Sub(place, value) => format!("sub {}, {}", place.nasm(), value.nasm()),
            Op::Add(place, value) => format!("sub {}, {}", place.nasm(), value.nasm()),
        }
    }
}

#[derive(Debug)]
pub struct LBB {
    pub label: &'static str,
    pub ops: Vec<Op>,
}

impl Codegen for LBB {
    fn nasm(&self) -> String {
        let mut buf = String::new();
        for op in &self.ops {
            buf.push_str(&*op.nasm());
            buf.push('\n');
        }
        buf
    }
}
