use glair::amd64;
use glair::linux64;
use glair::rtl;
use std::mem;

fn main() {
    let instructions = vec![
        rtl::RtlOp::Move(rtl::Move {
            dest: rtl::Register::Phys(rtl::PhysicalReg::Amd64(amd64::RCX)),
            value: rtl::RValue::Immediate(rtl::Immediate::I32(69)),
        }),
        rtl::RtlOp::Return(rtl::Return {
            value: Some(rtl::Register::Phys(rtl::PhysicalReg::Amd64(amd64::RCX))),
            cc: rtl::CallingConvention::C,
        }),
    ];

    let code: Vec<u8> = instructions
        .iter()
        .filter_map(|op| op.compile_amd64().ok())
        .flat_map(|instr| instr)
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
