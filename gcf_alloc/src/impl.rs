use crate::{GAInstrNo, GAOperandNo};

impl GAOperandNo {
    pub fn opr_no(&self) -> u32 {
        self.0
    }
}

impl GAInstrNo {
    pub fn instr_no(&self) -> u32 {
        self.0
    }
}
