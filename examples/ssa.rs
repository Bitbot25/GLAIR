use compile::CompileIntoBlock;
use glair::compile;
use glair::rtl;
use glair::ssa;

fn main() {
    let mut bb = ssa::BasicBlock::new();
    let mut sv = ssa::GLIRSupervisor::new();
    let mut emitter = bb.emitter(&mut sv);
    let x_0 = emitter.emit_cpy(ssa::RValue::Lit(ssa::Literal::U32(10)));
    let y_0 = emitter.emit_cpy(ssa::RValue::Var(x_0));
    let _y_1 = emitter.emit_binop(
        ssa::RValue::Var(y_0),
        ssa::RValue::Var(x_0),
        ssa::BinOpTy::Sub,
    );

    let mut compiled_rtl = bb.compile_into_block();
    let (regs, occupied) = compile::ralloc::analyze_rtl(&compiled_rtl.ops);
    let mut allocator =
        compile::ralloc::Allocator::new(regs.entries().map(|(k, v)| (k, *v)).collect(), occupied);
    allocator.create_allocations();
    let map = allocator.map();
    dbg!(map);

    // let mut codegen_ctx = codegen::CodegenContext::default();
    rtl::promote_registers_in_ops(&mut compiled_rtl.ops, |vir| {
        map.get(vir)
            .map(|alloc| alloc.kind)
            .expect("unmapped register")
    });

    println!("{}", compiled_rtl);
}
