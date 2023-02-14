use crate::rtx::{DestinationExpr, Rtx};

pub struct Transfer {
    destination: DestinationExpr,
    source: Rtx,
}

impl Transfer {
    pub fn new(destination: DestinationExpr, source: Rtx) -> Self {
        Transfer {
            destination,
            source,
        }
    }

    pub fn destination(&self) -> &DestinationExpr {
        &self.destination
    }

    pub fn source(&self) -> &Rtx {
        &self.source
    }
}

pub struct Return;

pub enum Instruction {
    Transfer(Transfer),
    Return(Return),
}
