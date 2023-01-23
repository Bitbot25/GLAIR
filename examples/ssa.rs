use glair::linux64;
use glair::rtl;
use std::mem;

fn main() {
    let mut instructions = vec![
        rtl::RtlOp::Move(rtl::Move {
            dest: rtl::Register::Phys(rtl::PhysicalReg::Stack),
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
    let mut stack = rtl::StackAllocator::default();
    let changes =
        rtl::RegisterAllocator::perform_allocations_and_modify(&mut instructions, &mut stack);
    dbg!(changes);

    let ctx = rtl::Context::new(Some(rtl::FunctionFootprint {
        stack_size: stack.top(),
    }));

    let code: Vec<u8> = instructions
        .iter()
        .filter_map(|op| op.compile_amd64(&ctx).ok())
        .flat_map(|instr| instr)
        .collect();
    for b in &code {
        eprintln!("{0:8b} | {0:x}", b);
    }
    /*#[rustfmt::skip]
    let code: Vec<u8> = vec![
        0x48, 0x83, 0xec, 0x08,       // sub rsp, 8
        0x89, 0x0c, 0x24,             // mov [rsp], ecx
        0x48, 0x83, 0xc4, 0x08,       // add rsp, 8
        0xb8, 0x45, 0x00, 0x00, 0x00, // mov eax, 0x45
        0xc3                          // ret near
    ];

    eprintln!("-----------------");
    for b in &code {
        eprintln!("{0:8b} | {0:x}", b);
    }*/

    let buf = linux64::MMapHandle::executable(code.as_slice());

    type FuncTy = unsafe extern "C" fn(*mut u32) -> u64;
    let func: FuncTy = unsafe { mem::transmute::<*mut u8, FuncTy>(buf.raw()) };
    let mut placeholder = Box::new(0u32);
    let result = unsafe { func(placeholder.as_mut() as *mut u32) };

    dbg!(result);
}
