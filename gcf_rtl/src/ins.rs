use crate::rtx::{DestinationExpr, Rtx};

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Return;

#[derive(Debug)]
pub enum Instruction {
    Transfer(Transfer),
    Return(Return),
}
