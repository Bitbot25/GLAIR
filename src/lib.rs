pub mod il;
#[cfg(all(target_os = "linux", target_pointer_width = "64"))]
pub mod linux64;

pub enum Arch {
    Amd64,
}
