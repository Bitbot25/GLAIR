use burnerflame::{AssmMov, AssmRet};
use glair::il;
use glair::il::cfg;
use glair::linux64;
use std::mem;

fn main() {
    let instructions = vec![
        il::Instruction::Write(il::Write {
            destination: il::SSARegister::MachineRegister(il::MachineReg::AMD64(
                burnerflame::Register::EAX,
            )),
            value: il::RValue::Immediate(il::Immediate::U32(16)),
        }),
        il::Instruction::Return(il::Return { register: None }),
    ];
    let mut cfg = cfg::CtrlFlow::new();
    cfg.insert_block(cfg::Block::new(instructions));

    let mut assembler = burnerflame::Assembler::new();
    //for instruction in instructions {
    //    instruction.generate_amd64(&mut assembler);
    //}
    assembler.mov(burnerflame::Register32::eax(), 16u32);
    assembler.ret();
    let code: Vec<u8> = assembler.into_buf();
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

    type FuncTy = unsafe extern "C" fn() -> u32;
    let func: FuncTy = unsafe { mem::transmute::<*mut u8, FuncTy>(buf.raw()) };
    let result = unsafe { func() };

    dbg!(result);
}
