pub mod amd64;
pub mod cfg;
#[cfg(all(target_os = "linux", target_pointer_width = "64"))]
pub mod linux64;
pub mod rtl;

pub enum Arch {
    Amd64,
}
