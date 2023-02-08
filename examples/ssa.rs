use glair::galloc::ifr;
use glair::galloc::liveness;
use glair::il;
use glair::il::amd;
use glair::il::cfg;
use glair::il::reg;

fn main() {
    let mut cfg = cfg::CtrlFlow::new();

    let r0 = reg::SSARegister::new(
        0,
        il::ILSize::Integer {
            width_in_bytes: 32 / 8,
        },
    );
    let r1 = reg::SSARegister::new(
        1,
        il::ILSize::Integer {
            width_in_bytes: 32 / 8,
        },
    );

    let entry_block = cfg.insert_block(cfg::Block::new(vec![
        il::Instruction::DummyUse(il::DummyUse { register: r1 }),
        il::Instruction::Write(il::Write {
            destination: r0,
            value: il::RValue::Immediate(il::Immediate::U32(16)),
        }),
        il::Instruction::DummyUse(il::DummyUse { register: r0 }),
        il::Instruction::Return(il::Return { register: None }),
    ]));
    let other_block = cfg.insert_block(cfg::Block::new(vec![il::Instruction::DummyUse(
        il::DummyUse { register: r0 },
    )]));
    cfg.add_directed_edge(entry_block, other_block);

    let vars = &[r0, r1];
    let mut range_builder = liveness::LiveRangesBuilder::default();

    for var in vars {
        let deaths = liveness::find_deaths(var, entry_block, &cfg);
        dbg!(&deaths);
        for death in &deaths {
            liveness::mark_live_in_range(
                var,
                cfg::Location::new(entry_block, 0),
                *death,
                &mut range_builder,
                &cfg,
            );
        }
    }
    let mut ifr_graph = ifr::construct(range_builder.build());
    ifr::dsatur(&mut ifr_graph);
    dbg!(ifr_graph);

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
