pub mod amd64;
#[cfg(all(target_os = "linux", target_pointer_width = "64"))]
pub mod linux64;
