use crate::rtl::{amd64::Amd64Register, Op, Ops, RValue, RealRegister, Register, VirRegister};
use std::fmt;

const AMD64_ALLOC_DWORD_REG: [Amd64Register; 4] = [
    Amd64Register::Eax,
    Amd64Register::Ebx,
    Amd64Register::Ecx,
    Amd64Register::Edx,
];

pub struct VirRegisterMap<T> {
    arr: Vec<Option<(T, usize)>>,
}

impl<T> VirRegisterMap<T> {
    pub fn new() -> VirRegisterMap<T> {
        VirRegisterMap { arr: Vec::new() }
    }

    pub fn insert(&mut self, vir: &VirRegister, val: T) {
        self.expand_to(vir.n + 1);
        self.arr[vir.n] = Some((val, vir.bytes));
    }

    pub fn get(&self, vir: &VirRegister) -> Option<&T> {
        if vir.n >= self.arr.len() {
            None
        } else {
            match &self.arr[vir.n] {
                Some((val, _bytes)) => Some(val),
                None => None,
            }
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = &T> + '_ {
        self.arr.iter().filter_map(|x| match x {
            Some((t, _bytes)) => Some(t),
            None => None,
        })
    }

    pub fn entries(&self) -> impl Iterator<Item = (VirRegister, &T)> + '_ {
        self.arr.iter().enumerate().filter_map(|(n, e)| match e {
            Some((t, bytes)) => Some((VirRegister { n, bytes: *bytes }, t)),
            None => None,
        })
    }

    fn expand_to(&mut self, n: usize) {
        if self.arr.len() < n {
            let diff = n - self.arr.len();
            self.arr.reserve(diff);
            for _ in 0..diff {
                self.arr.push(None);
            }
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for VirRegisterMap<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VirRegisterMap [\n")?;
        // TODO: Indentation (PadAdapter)
        for val in &self.arr {
            match val {
                Some(entry) => {
                    fmt::Debug::fmt(entry, f)?;
                    write!(f, "\n")?;
                }
                None => (),
            }
        }
        write!(f, "]")
    }
}

#[derive(Debug)]
pub struct Allocator {
    allocations: VirRegisterMap<Allocation>,
    virtuals: Vec<(VirRegister, VirRegisterInfo)>,
    manually_excluded: Vec<RealRegister>,
}

impl Allocator {
    pub fn new(
        virtuals: Vec<(VirRegister, VirRegisterInfo)>,
        occupied: Vec<RealRegister>,
    ) -> Allocator {
        Allocator {
            virtuals,
            manually_excluded: occupied,
            allocations: VirRegisterMap::new(),
        }
    }

    #[inline]
    pub fn map(&self) -> &VirRegisterMap<Allocation> {
        &self.allocations
    }

    pub fn create_allocations(&mut self) {
        for (vir, info) in &self.virtuals {
            assert_eq!(vir.bytes, 4, "only support for 4 bytes");
            // FIXME: Support for other than AMD64
            let allocations: Vec<&Allocation> = self.allocations.keys().collect();
            let mut choices: Vec<&Amd64Register> = AMD64_ALLOC_DWORD_REG
                .iter()
                .filter(|reg| {
                    !self.manually_excluded.contains(&RealRegister::Amd64(**reg))
                        && !allocations.iter().any(|alloc| match &alloc.reg {
                            RealRegister::Amd64(amd) => amd == *reg,
                        })
                })
                .collect();
            let reg = choices.drain(..).next().expect("no free registers!");

            self.allocations.insert(
                vir,
                Allocation {
                    info: *info,
                    vir: *vir,
                    reg: RealRegister::Amd64(*reg),
                },
            );
        }
    }
}

pub fn analyze_rtl(ops: &Ops) -> (VirRegisterMap<VirRegisterInfo>, Vec<RealRegister>) {
    let mut map: VirRegisterMap<VirRegisterInfo> = VirRegisterMap::new();
    let mut already_occupied: Vec<RealRegister> = Vec::new();

    fn notice_reg(
        map: &mut VirRegisterMap<VirRegisterInfo>,
        already_occupied: &mut Vec<RealRegister>,
        reg: &Register,
        idx: InstrIdx,
    ) {
        let vir_reg = match reg {
            Register::Real(real) => {
                already_occupied.push(*real);
                return;
            }
            Register::Vir(vir) => vir,
        };
        match map.get(vir_reg) {
            None => map.insert(
                vir_reg,
                VirRegisterInfo {
                    lifetime_begin: idx,
                    lifetime_end: idx,
                },
            ),
            Some(info) => map.insert(
                vir_reg,
                VirRegisterInfo {
                    lifetime_begin: info.lifetime_begin,
                    lifetime_end: idx,
                },
            ),
        }
    }

    #[inline]
    fn notice_rvalue(
        map: &mut VirRegisterMap<VirRegisterInfo>,
        already_occupied: &mut Vec<RealRegister>,
        rv: &RValue,
        idx: InstrIdx,
    ) {
        match rv {
            RValue::Lit(..) => (),
            RValue::Register(reg) => notice_reg(map, already_occupied, reg, idx),
        }
    }

    macro_rules! notice_both {
        ($map:expr,$occ:expr,$tgt:ident,$fa:ident,$fb:ident,$i:expr) => {{
            notice_reg(&mut $map, &mut $occ, &$tgt.$fa, $i);
            notice_rvalue(&mut $map, &mut $occ, &$tgt.$fb, $i);
        }};
    }

    for (i, op) in ops.iter().enumerate() {
        match op {
            Op::Add(add) => notice_both!(map, already_occupied, add, to, val, i),
            Op::Sub(sub) => notice_both!(map, already_occupied, sub, from, val, i),
            Op::Mul(mul) => notice_both!(map, already_occupied, mul, val, with, i),
            Op::Div(div) => notice_both!(map, already_occupied, div, val, with, i),
            Op::Copy(copy) => notice_both!(map, already_occupied, copy, to, from, i),
        }
    }
    (map, already_occupied)
}

#[derive(Copy, Clone, Debug)]
pub struct VirRegisterInfo {
    lifetime_begin: InstrIdx,
    lifetime_end: InstrIdx,
}

impl Allocator {
    /*
    fn allocate_registers(registers: Vec<(VirRegister, VirRegisterInfo)>) -> Vec<> {

    }
    */
}

#[derive(Debug)]
pub struct Allocation {
    pub info: VirRegisterInfo,
    pub vir: VirRegister,
    pub reg: RealRegister,
}

type InstrIdx = usize;
