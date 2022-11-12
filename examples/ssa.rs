use compile::CompileIntoBlock;
use glair::compile;
use glair::rtl;
use glair::ssa;
use glair::typing;

fn main() {
    let mut sv = ssa::GLIRSupervisor::new();
    let x_0 = sv.create_var(typing::Type::U32);
    let y_0 = sv.create_var(typing::Type::U32);
    let y_1 = sv.create_descendant(y_0);

    let mut bb = ssa::BasicBlock::new();
    let mut emitter = bb.emitter();
    emitter.emit_lit(x_0, ssa::Literal::U32(10));
    emitter.emit_cpy(y_0, ssa::RValue::Var(x_0));
    emitter.emit_binop(
        y_1,
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
            .map(|alloc| alloc.reg)
            .expect("unmapped register")
    });

    println!("{}", compiled_rtl);
}
