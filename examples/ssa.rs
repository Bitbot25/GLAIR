use glair::amd64;
use glair::linux64;
use glair::rtl;
use std::mem;

fn main() {
    let mut instructions = vec![
        // reduntant move
        rtl::RtlOp::Move(rtl::Move {
            dest: rtl::Register::Virtual(rtl::VirtualReg {
                id: 1,
                data_ty: rtl::RegDataType::Int32,
            }),
            value: rtl::RValue::Immediate(rtl::Immediate::I32(420)),
        }),
        // TODO: Panic when type of register and value is not the same. It is OK for amd64, because you can move 32-bit value into 64-bit registers, but that may not be the case for other IAs.
        rtl::RtlOp::Move(rtl::Move {
            dest: rtl::Register::Virtual(rtl::VirtualReg {
                id: 0,
                data_ty: rtl::RegDataType::Int64,
            }),
            value: rtl::RValue::Immediate(rtl::Immediate::I32(69)),
        }),
        rtl::RtlOp::Return(rtl::Return {
            value: Some(rtl::Register::Virtual(rtl::VirtualReg {
                id: 0,
                data_ty: rtl::RegDataType::Int64,
            })),
            cc: rtl::CallingConvention::C,
        }),
    ];
    let changes = rtl::RegisterAllocator::perform_allocations_and_modify(&mut instructions);
    dbg!(changes);

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
