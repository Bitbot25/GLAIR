pub mod typing {
    #[derive(Hash, Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Type {
        I32,
    }

    impl Type {
        pub const fn mem_size(&self) -> usize {
            match self {
                Type::I32 => 4,
            }
        }
    }

    pub trait Typed {
        fn typ(&self) -> Type;
    }
}

pub mod ssa {
    use super::typing::{self, Typed};
    use std::fmt;

    #[derive(Hash, Debug, Copy, Clone, PartialEq, Eq)]
    pub struct Variable {
        ver: usize,
        name: &'static str,
        typ: typing::Type,
    }

    impl Variable {
        pub fn new(name: &'static str, typ: typing::Type) -> Variable {
            Variable { name, ver: 0, typ }
        }

        pub fn ssa_bump(&self) -> Variable {
            Variable {
                name: self.name,
                ver: self.ver + 1,
                typ: self.typ,
            }
        }
    }

    impl typing::Typed for Variable {
        fn typ(&self) -> typing::Type {
            self.typ
        }
    }

    impl fmt::Display for Variable {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}_{}", self.name, self.ver)
        }
    }

    #[derive(Debug)]
    pub enum InlineValue {
        I32(i32),
    }

    impl typing::Typed for InlineValue {
        fn typ(&self) -> typing::Type {
            match self {
                InlineValue::I32(..) => typing::Type::I32,
            }
        }
    }

    impl fmt::Display for InlineValue {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                InlineValue::I32(val) => write!(f, "{}", *val),
            }
        }
    }

    #[derive(Debug)]
    pub enum Operand {
        Inline(InlineValue),
        Variable(Variable),
    }

    impl typing::Typed for Operand {
        #[inline]
        fn typ(&self) -> typing::Type {
            match self {
                Operand::Inline(inline) => inline.typ(),
                Operand::Variable(var) => var.typ(),
            }
        }
    }

    impl Operand {
        #[inline]
        pub fn mem_size(&self) -> usize {
            self.typ().mem_size()
        }
    }

    impl fmt::Display for Operand {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Operand::Inline(inline) => fmt::Display::fmt(inline, f),
                Operand::Variable(var) => fmt::Display::fmt(var, f),
            }
        }
    }
}

#[derive(Debug)]
pub enum Ins {
    Sub(
        ssa::Variable,
        /* = */ ssa::Operand,
        /* - */ ssa::Operand,
    ),
    Init(ssa::Variable, ssa::Operand),
}

pub mod bb {
    use super::{ssa, Ins};

    #[derive(Debug)]
    pub struct BasicBlock {
        pub ins_list: Vec<Ins>,
        pub terminator: Terminator,
    }

    #[derive(Debug)]
    #[allow(dead_code)]
    pub enum Terminator {
        Ret(ssa::Variable),
        Jmp(Box<BasicBlock>),
        Void,
    }
}
