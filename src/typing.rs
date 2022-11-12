#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    I32,
    U32,
}

impl Type {
    pub const fn mem_size(&self) -> usize {
        match self {
            Type::I32 => 4,
            Type::U32 => 4,
        }
    }
}

pub trait Typed {
    fn data_ty(&self) -> Type;
}
