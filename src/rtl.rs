// TODO: Change this later to be more "generic".
pub enum Reg {
    Eax,
}

impl Codegen for Reg {
    fn nasm(&self) -> String {
        match self {
            Reg::Eax => "eax".to_string(),
        }
    }
}

pub trait Codegen {
    fn nasm(&self) -> String;
}

pub enum StackType {
    DWord, // i32
}

impl Codegen for StackType {
    fn nasm(&self) -> String {
        match self {
            StackType::DWord => "dword".to_string(),
        }
    }
}

pub enum Place {
    Stack { typ: StackType, sp_offset: usize },
    Register(Reg)
}

impl Codegen for Place {
    fn nasm(&self) -> String {
        match self {
            Place::Stack { typ, sp_offset } => format!("{} [esp-{}]", typ.nasm(), sp_offset),
            Place::Register(reg) => reg.nasm(),
        }
    }
}

pub enum Value {
    I32(i32),
}

impl Codegen for Value {
    fn nasm(&self) -> String {
        match self {
            Value::I32(val) => val.to_string()
        }
    }
}

pub enum Op {
    Move(Place, Value),
    Sub(Place, Value),
}

impl Codegen for Op {
    fn nasm(&self) -> String {
        match self {
            Op::Move(place, value) => format!("mov {}, {}", place.nasm(), value.nasm()),
            Op::Sub(..) => todo!()
        }
    }
}

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
