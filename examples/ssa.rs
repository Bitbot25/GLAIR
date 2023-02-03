use burnerflame::{AssmMov, AssmRet};
use glair::galloc::liveness;
use glair::il;
use glair::il::cfg;
use glair::linux64;
use std::mem;

fn main() {
    let mut cfg = cfg::CtrlFlow::new();

    let eax = il::SSARegister::machine_reg(0, il::MachineReg::AMD64(burnerflame::Register::EAX));
    let ecx = il::SSARegister::machine_reg(1, il::MachineReg::AMD64(burnerflame::Register::ECX));
    let entry_block = cfg.insert_block(cfg::Block::new(vec![
        il::Instruction::Write(il::Write {
            destination: eax,
            value: il::RValue::Immediate(il::Immediate::U32(16)),
        }),
        il::Instruction::DummyUse(il::DummyUse { register: eax }),
        il::Instruction::DummyUse(il::DummyUse { register: ecx }),
        il::Instruction::Return(il::Return { register: None }),
    ]));
    let other_block = cfg.insert_block(cfg::Block::new(vec![il::Instruction::DummyUse(
        il::DummyUse { register: eax },
    )]));
    cfg.add_directed_edge(entry_block, other_block);

    let deaths = liveness::find_deaths(&eax, entry_block, &cfg);
    dbg!(&deaths);
    for death in deaths {
        liveness::mark_live_in_range(
            &eax,
            cfg::Location::new(entry_block, 0),
            death,
            |_var, _ins, _loc| false,
            |var, ins, loc| eprintln!("live mark {var:?} at {loc:?}"),
            &cfg,
        );
    }

    /*let mut assembler = burnerflame::Assembler::new();
    for instruction in block.realise(&cfg).instructions() {
        instruction.generate_amd64(&mut assembler);
    }
    let code: Vec<u8> = assembler.into_buf();
    for b in &code {
        eprintln!("{0:8b} | {0:x}", b);
    }
    let buf = linux64::MMapHandle::executable(code.as_slice());

    type FuncTy = unsafe extern "C" fn() -> u32;
    let func: FuncTy = unsafe { mem::transmute::<*mut u8, FuncTy>(buf.raw()) };
    let result = unsafe { func() };

    dbg!(result);*/
}
