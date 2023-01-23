mod impl_amd;
mod impl_misc;

pub enum SizeSpecification {
    Pointer,
    Integer { width_in_bytes: usize },
    Structure { width_in_bytes: usize },
}

pub struct PlaceholderReg {
    identifier: usize,
    size: SizeSpecification,
}

pub enum MachineReg {
    AMD64(burnerflame::Register),
}

pub enum SSARegister {
    Placeholder(PlaceholderReg),
    MachineRegister(MachineReg),
}

/// Allocates memory on the stack and places the pointer on [`ptr_out`]
pub struct Reserve {
    size: SizeSpecification,
    out_pointer: SSARegister,
}

/// Writes [`value`] to [`destination`]
pub struct Write {
    destination: SSARegister,
    value: SSARegister,
}

/// Reads the data pointed to by [`target`] into [`out_data`]
pub struct Read {
    target: SSARegister,
    out_data: SSARegister,
}

/// IL Instruction
pub enum Instruction {
    Reserve(Reserve),
    Write(Write),
    Read(Read),
}
