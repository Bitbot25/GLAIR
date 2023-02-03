#![feature(type_alias_impl_trait)]

pub mod galloc;
pub mod il;
#[cfg(all(target_os = "linux", target_pointer_width = "64"))]
pub mod linux64;
