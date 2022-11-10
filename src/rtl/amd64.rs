#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Amd64Memory {
    Register(usize, Amd64Register),
    Addr(usize, isize),
    Add(Box<(Amd64Memory, Amd64Memory)>),
    Sub(Box<(Amd64Memory, Amd64Memory)>),
}

impl Amd64Memory {
    pub fn sz(&self) -> usize {
        match self {
            Amd64Memory::Register(sz, _reg) => *sz,
            Amd64Memory::Addr(sz, _addr) => *sz,
            Amd64Memory::Add(operands) => {
                let sz_a = operands.0.sz();
                let sz_b = operands.1.sz();
                assert_eq!(sz_a, sz_b);
                sz_a
            }
            Amd64Memory::Sub(operands) => {
                let sz_a = operands.0.sz();
                let sz_b = operands.1.sz();
                assert_eq!(sz_a, sz_b);
                sz_a
            }
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum Amd64Register {
    Eax,
    Ecx,
    Esp,

    Rax,
    Rcx,
    Rsp,
}

impl Amd64Register {
    pub fn name(&self) -> &'static str {
        match self {
            Amd64Register::Eax => "eax",
            Amd64Register::Ecx => "ecx",
            Amd64Register::Esp => "esp",
            Amd64Register::Rax => "rax",
            Amd64Register::Rcx => "rcx",
            Amd64Register::Rsp => "rsp",
        }
    }

    pub fn sz(&self) -> usize {
        match self {
            Amd64Register::Eax | Amd64Register::Ecx | Amd64Register::Esp => 4,
            Amd64Register::Rax | Amd64Register::Rcx | Amd64Register::Rsp => 8,
        }
    }
}
