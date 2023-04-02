use crate::{GAInstrNo, GALiveWhen, GALocation, GAOperandNo, GARegNo, GAVarNo, LRSlice, LiveRange};
use std::fmt::Formatter;
use std::fmt::Result;
use std::fmt::{Debug, Display};

impl Display for GAOperandNo {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        write!(fmt, "opr({})", self.0)
    }
}

impl Display for GAVarNo {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        write!(fmt, "var({})", self.0)
    }
}

impl Display for GARegNo {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        write!(fmt, "reg({})", self.0)
    }
}

impl Display for GAInstrNo {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        write!(fmt, "instr({})", self.0)
    }
}

impl Display for GALocation {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        match self.when {
            GALiveWhen::Pre => write!(fmt, "before({})", self.instr.instr_no()),
            GALiveWhen::Post => write!(fmt, "after({})", self.instr.instr_no()),
        }
    }
}

impl Display for LRSlice {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        write!(fmt, "{}..{}", self.from, self.to)
    }
}

impl Display for LiveRange {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        fmt.debug_list()
            .entries(self.slices.iter().map(|x| DisplayToDebug(x)))
            .finish()
    }
}

struct DisplayToDebug<T: Display>(T);
impl<T: Display> Debug for DisplayToDebug<T> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        write!(fmt, "{}", self.0)
    }
}
