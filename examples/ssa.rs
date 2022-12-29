use glair::amd64;
use glair::linux64;
use std::mem;

fn main() {
    let instructions = vec![
        amd64::OpCode::MovRegImm32(amd64::MovRegImm32 {
            reg: amd64::RCX,
            imm: amd64::Imm32 { int32: 10 },
        }),
        amd64::OpCode::MovRegReg(amd64::MovRegReg {
            dest: amd64::RAX,
            value: amd64::RCX,
        }),
        /*amd64::OpCode::Mov(amd64::MovGeneric {
            destination: amd64::RegMem::Reg(amd64::ECX),
            value: amd64::RegImm::Imm(amd64::Immediate::Imm32(amd64::Imm32 { int32: 10 })),
        }),
        amd64::OpCode::Mov(amd64::MovGeneric {
            destination: amd64::RegMem::Reg(amd64::EAX),
            value: amd64::RegImm::Reg(amd64::ECX),
        }),*/
        amd64::OpCode::RetNear,
    ];
    let code: Vec<u8> = instructions
        .iter()
        .flat_map(|op| op.compile_amd64())
        .collect();

    for b in &code {
        eprintln!("{0:8b} | {0:x}", b);
    }

    let buf = linux64::MMapHandle::executable(code.as_slice());

    type FuncTy = unsafe extern "C" fn() -> u64;
    let func: FuncTy = unsafe { mem::transmute::<*mut u8, FuncTy>(buf.raw()) };
    let result = unsafe { func() };

    dbg!(result);
}
