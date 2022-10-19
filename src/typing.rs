
    use crate::rtl;

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

    impl rtl::AsWordTy for Type {
        fn word_ty(&self) -> rtl::WordTy {
            match self.mem_size() {
                4 => rtl::WordTy::DWord,
                _ => unreachable!(),
            }
        }
    }

    pub trait Typed {
        fn typ(&self) -> Type;
    }
