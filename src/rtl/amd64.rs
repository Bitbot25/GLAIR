#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Copy, Clone)]
pub enum Amd64Register {
    Eax,
    Ebx,
    Ecx,
    Edx,
    Esp,

    Rax,
    Rcx,
    Rsp,
}

impl Amd64Register {
    pub fn name(&self) -> &'static str {
        match self {
            Amd64Register::Eax => "eax",
            Amd64Register::Ebx => "ebx",
            Amd64Register::Ecx => "ecx",
            Amd64Register::Edx => "edx",
            Amd64Register::Esp => "esp",
            Amd64Register::Rax => "rax",
            Amd64Register::Rcx => "rcx",
            Amd64Register::Rsp => "rsp",
        }
    }

    pub fn reg_size(&self) -> usize {
        match self {
            Amd64Register::Eax
            | Amd64Register::Ebx
            | Amd64Register::Ecx
            | Amd64Register::Edx
            | Amd64Register::Esp => 4,
            Amd64Register::Rax | Amd64Register::Rcx | Amd64Register::Rsp => 8,
        }
    }
}
