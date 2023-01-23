use std::collections::HashMap;

use crate::amd64;
use crate::cfg;

#[derive(Debug)]
pub struct RegisterStatistics<'a> {
    points: Vec<&'a mut Register>,
}

struct Analysis<'a> {
    register_stats: Vec<(VirtualReg, RegisterStatistics<'a>)>,
    preoccupied_regs: Vec<&'a PhysicalReg>,
}

fn analyze<'a>(code: &'a mut Vec<RtlOp>) -> Analysis<'a> {
    fn do_insert<'a>(
        map: &mut HashMap<VirtualReg, RegisterStatistics<'a>>,
        point_wrap: &'a mut Register,
    ) {
        let point_unwrap = match point_wrap {
            Register::Virtual(vir) => vir,
            Register::Phys(_) => unreachable!(),
        };
        let exist = map.get_mut(point_unwrap);
        match exist {
            Some(stats) => stats.points.push(point_wrap),
            None => {
                map.insert(
                    *point_unwrap,
                    RegisterStatistics {
                        points: vec![point_wrap],
                    },
                );
            }
        };
    }

    let mut map: HashMap<VirtualReg, RegisterStatistics<'a>> = HashMap::new();
    let mut preoccupied_regs: Vec<&PhysicalReg> = Vec::new();
    for op in code {
        match op {
            RtlOp::Move(Move { dest, value }) => {
                match dest {
                    Register::Virtual(_vir) => do_insert(&mut map, dest),
                    Register::Phys(phys) => preoccupied_regs.push(phys),
                };
                match value {
                    RValue::Register(reg @ Register::Virtual(_)) => do_insert(&mut map, reg),
                    RValue::Register(Register::Phys(phys)) => preoccupied_regs.push(phys),
                    RValue::Immediate(_) => (),
                };
            }
            RtlOp::Return(Return { value, .. }) => match value {
                Some(vir @ Register::Virtual(_)) => do_insert(&mut map, vir),
                Some(Register::Phys(phys)) => preoccupied_regs.push(phys),
                None => (),
            },
            RtlOp::FixAlignment(_) => (),
            RtlOp::Call(Call { callee: _ }) => todo!(),
        }
    }
    Analysis {
        register_stats: map.into_iter().collect(),
        preoccupied_regs,
    }
}

#[derive(Debug)]
pub struct RegisterAllocator {
    mappings: HashMap<VirtualReg, PhysicalReg>,
}

#[derive(Default)]
pub struct StackAllocator {
    offset: usize,
}

impl StackAllocator {
    pub fn allocate(&mut self, data_ty: RegDataType) -> PhysicalReg {
        let size_in_bytes = match data_ty {
            RegDataType::Custom(sz) => {
                validate_alignment_amd64(sz); // FIXME: Validate alignment for other IAs
                sz
            }
            RegDataType::Int8 => 8, // All the sizes are 8 because of the stack alignment (FIXME: This may be different for other IAs)
            RegDataType::Int16 => 8,
            RegDataType::Int32 => 8,
            RegDataType::Int64 => 8,
        };
        self.offset += size_in_bytes;
        let v = PhysicalReg::Stack(StackRegister {
            scope_slot: self.offset,
            data_ty,
        });
        v
    }

    pub fn top(&self) -> usize {
        self.offset
    }
}

impl RegisterAllocator {
    pub fn perform_allocations_and_modify(
        code_block: &mut Vec<RtlOp>,
        stack: &mut StackAllocator,
    ) -> Self {
        let Analysis {
            register_stats,
            preoccupied_regs,
        } = analyze(code_block);

        let mut priority = Vec::new();
        for (reg, stats) in &register_stats {
            let RegisterStatistics { points } = stats;
            let count = points.len();
            priority.push((count, reg));
        }
        priority.sort_unstable_by(|(a_count, _), (b_count, _)| b_count.cmp(a_count));
        // More used regisers will come first.
        let priority: Vec<&VirtualReg> = priority.into_iter().map(|(_, reg)| reg).collect();
        dbg!(&priority);
        fn registers(filter_out: &Vec<PhysicalReg>) -> impl Iterator<Item = PhysicalReg> + '_ {
            amd64::generic_registers()
                .into_iter()
                .map(|x| *x)
                .map(PhysicalReg::Amd64)
                .filter(|reg| !filter_out.contains(reg))
        }
        let mut unavailable_phys_reg: Vec<PhysicalReg> =
            preoccupied_regs.into_iter().map(|r| *r).collect();

        fn find_phys_reg(
            data_ty: RegDataType,
            regs: impl Iterator<Item = PhysicalReg>,
        ) -> Option<PhysicalReg> {
            for reg in regs {
                if reg.data_ty() == data_ty {
                    return Some(reg);
                }
            }
            return None;
        }

        let mut map = HashMap::new();
        for reg in priority {
            let phys_reg = match find_phys_reg(reg.data_ty, registers(&unavailable_phys_reg)) {
                Some(x) => x,
                None => stack.allocate(reg.data_ty),
            };
            match phys_reg {
                PhysicalReg::Amd64(amd_reg) => {
                    // Remove sub- and parent registers aswell.
                    for reg in amd_reg.relatives() {
                        unavailable_phys_reg.push(PhysicalReg::Amd64(*reg));
                    }
                }
                _ => {
                    unavailable_phys_reg.push(phys_reg);
                }
            }
            map.insert(*reg, phys_reg);
        }

        // Replace virtual registers
        for (reg, stats) in register_stats {
            for point in stats.points {
                *point = Register::Phys(map[&reg]);
            }
        }

        RegisterAllocator { mappings: map }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum RegDataType {
    Int8,
    Int16,
    Int32,
    Int64,
    Custom(usize),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct VirtualReg {
    pub id: u32,
    pub data_ty: RegDataType,
}

pub trait ContainsDataType {
    fn data_ty(&self) -> RegDataType;
}

#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy)]
pub struct StackRegister {
    scope_slot: usize,
    data_ty: RegDataType,
}

#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy)]
pub enum PhysicalReg {
    Amd64(amd64::Reg),
    Stack(StackRegister),
}

impl ContainsDataType for PhysicalReg {
    fn data_ty(&self) -> RegDataType {
        match self {
            PhysicalReg::Amd64(r) => r.data_ty(),
            PhysicalReg::Stack(stack) => stack.data_ty,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Register {
    Virtual(VirtualReg),
    Phys(PhysicalReg),
}

impl ContainsDataType for Register {
    fn data_ty(&self) -> RegDataType {
        match self {
            Register::Virtual(vir) => vir.data_ty,
            Register::Phys(phys) => phys.data_ty(),
        }
    }
}

#[derive(Debug)]
pub enum Immediate {
    I32(i32),
}

#[derive(Debug)]
pub enum RValue {
    Immediate(Immediate),
    Register(Register),
}

#[derive(Debug)]
pub struct Move {
    pub dest: Register,
    pub value: RValue,
}

pub enum CompileRtlError {
    VirtualRegister,
    WrongRegisterConversion {
        expected: crate::Arch,
        found: crate::Arch,
    },
}

pub struct FunctionFootprint {
    pub stack_size: usize,
}

#[derive(Default)]
pub struct Context {
    current_function: Option<FunctionFootprint>,
}

impl Context {
    pub fn new(current_function: Option<FunctionFootprint>) -> Self {
        Context { current_function }
    }
}

impl Move {
    fn register_to_amd64_native(reg: Register) -> Result<amd64::Reg, CompileRtlError> {
        let Register::Phys(dest) = reg else {
            return Err(CompileRtlError::VirtualRegister);
        };
        match dest {
            PhysicalReg::Stack(_stack_reg) => panic!("stack registers are not applicable"),
            PhysicalReg::Amd64(dest) => Ok(dest),
        }
    }

    pub fn compile_amd64(&self, ctx: &Context) -> Result<Vec<u8>, CompileRtlError> {
        Ok(match &self.value {
            RValue::Immediate(imm) => match imm {
                Immediate::I32(i32) => amd64::MovRegImm32 {
                    reg: Self::register_to_amd64_native(self.dest)?,
                    imm: amd64::Imm32 { int32: *i32 },
                }
                .compile_amd64(),
            },
            RValue::Register(Register::Phys(PhysicalReg::Amd64(amd_reg_value))) => {
                match self.dest {
                    Register::Virtual(_) => return Err(CompileRtlError::VirtualRegister),
                    Register::Phys(PhysicalReg::Amd64(amd_reg_dest)) => amd64::MovRegReg {
                        dest: amd_reg_dest,
                        value: *amd_reg_value,
                    }
                    .compile_amd64(),
                    Register::Phys(PhysicalReg::Stack(stack_dest)) => amd64::MovMemReg {
                        dest: amd64::Memory::Disp8(
                            amd64::SP,
                            ctx.current_function
                                .as_ref()
                                .expect("no access to stack data")
                                .stack_size as u8
                                - stack_dest.scope_slot as u8,
                        ),
                        value: *amd_reg_value,
                    }
                    .compile_amd64(),
                }
            }
            RValue::Register(reg) => amd64::MovRegReg {
                dest: Self::register_to_amd64_native(self.dest)?,
                value: Self::register_to_amd64_native(*reg)?,
            }
            .compile_amd64(),
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CallingConvention {
    C,
}

#[derive(Debug)]
pub struct Return {
    pub value: Option<Register>,
    pub cc: CallingConvention,
}

fn validate_alignment_amd64(size: usize) {
    // FIXME: SIMD instructions require 16-byte alignment
    if size % 8 != 0 || size == 0 {
        panic!("Invalid stack alignment");
    }
}

impl Return {
    pub fn compile_amd64(&self) -> Vec<u8> {
        assert_eq!(self.cc, CallingConvention::C);
        match self.value {
            Some(Register::Phys(PhysicalReg::Amd64(ret_v))) => {
                let mut buf = Vec::new();
                let ret_t = ret_v.data_ty();
                match ret_t {
                    RegDataType::Int8 | RegDataType::Int16 => todo!(),
                    RegDataType::Int32 => {
                        // Place into eax
                        buf.append(
                            &mut amd64::MovRegReg {
                                dest: amd64::EAX,
                                value: ret_v,
                            }
                            .compile_amd64(),
                        );
                    }
                    RegDataType::Int64 => {
                        // Place into rax
                        buf.append(
                            &mut amd64::MovRegReg {
                                dest: amd64::RAX,
                                value: ret_v,
                            }
                            .compile_amd64(),
                        );
                    }
                    RegDataType::Custom(custom_sz) => {
                        if custom_sz <= 8 {
                            let bits = custom_sz * 8;
                            let reg = if bits > 8 {
                                if bits > 16 {
                                    if bits > 32 {
                                        amd64::RAX
                                    } else {
                                        amd64::EAX
                                    }
                                } else {
                                    amd64::AX
                                }
                            } else {
                                amd64::A
                            };

                            buf.append(
                                &mut amd64::MovRegReg {
                                    dest: reg,
                                    value: ret_v,
                                }
                                .compile_amd64(),
                            );
                        } else {
                            validate_alignment_amd64(custom_sz);
                            panic!("no support for stack")
                        }
                    }
                };
                buf.push(amd64::RetNear.compile_amd64());
                buf
            }
            Some(Register::Phys(PhysicalReg::Stack(_stack_r))) => {
                panic!("no support for stack registers")
            }
            // TODO: vvvvvv
            Some(Register::Virtual(_vir)) => {
                panic!("cannot return virtual register. (TODO: Return error type instead)")
            }
            None => vec![amd64::RetNear.compile_amd64()], // FIXME: Clear return registers if they are "occupied". We need a data structure to keep track of this.
        }
    }
}

#[derive(Debug)]
pub struct Call<'cfg> {
    callee: Function<'cfg>,
}

impl<'cfg> Call<'cfg> {
    pub fn compile_amd64(&self) -> Vec<u8> {
        todo!()
    }
}

#[derive(Debug)]
pub struct Function<'cfg> {
    block: cfg::Block<'cfg, RtlOp<'cfg>>,
}

#[derive(Debug)]
pub struct FixAlignment;

impl FixAlignment {
    pub fn compile_amd64(&self) -> Vec<u8> {
        todo!()
    }
}

#[derive(Debug)]
pub enum RtlOp<'cfg> {
    FixAlignment(FixAlignment),
    Move(Move),
    Return(Return),
    Call(Call<'cfg>),
}

impl<'cfg> RtlOp<'cfg> {
    pub fn compile_amd64(&self, ctx: &Context) -> Result<Vec<u8>, CompileRtlError> {
        match self {
            RtlOp::Move(mov) => mov.compile_amd64(ctx),
            RtlOp::FixAlignment(align) => Ok(align.compile_amd64()),
            RtlOp::Return(ret) => Ok(ret.compile_amd64()),
            RtlOp::Call(call) => Ok(call.compile_amd64()),
        }
    }
}
