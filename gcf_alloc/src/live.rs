use gcf_bb as bb;
use std::fmt;
use smallvec::{smallvec, SmallVec};
use std::collections::{HashMap, HashSet, VecDeque};

use super::*;

#[derive(Hash, PartialEq, Eq, Debug)]
pub struct LocalLiveRange {
    pub(super) from: LiveLocation,
    pub(super) to: LiveLocation,
}

#[derive(Hash, PartialEq, Eq, Debug)]
pub struct LiveLocation {
    pub(super) mode: LiveLocationMode,
    pub(super) index: InstructionId,
}

#[derive(Hash, Debug, Copy, Clone, PartialEq, Eq)]
pub enum LiveLocationMode {
    Pre,
    Post,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct LiveRange {
    pub(super) ranges: Vec<LocalLiveRange>,
    pub(super) reg: VrReg,
}

#[derive(Clone)]
pub struct LiveSet {
    vars: HashSet<VrReg>,
}

impl fmt::Debug for LiveSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.vars, f)
    }
}

impl LiveSet {
    pub fn new() -> Self {
        LiveSet {
            vars: HashSet::new(),
        }
    }

    pub fn union_add(&mut self, other: &LiveSet) {
        for v in &other.vars {
            self.vars.insert(*v);
        }
    }

    pub fn set(&mut self, reg: VrReg, b: bool) {
        if b {
            self.vars.insert(reg);
        } else {
            self.vars.remove(&reg);
        }
    }

    pub fn contains(&self, reg: VrReg) -> bool {
        self.vars.contains(&reg)
    }

    pub fn iter(&self) -> impl Iterator<Item = VrReg> + '_ {
        self.vars.iter().map(|x| *x)
    }
}
