use gcf_rtl::{prelude as rtl, reg::AccessMode};
use gcf_x86_64 as x86_64;
use x86_64::BitMode;

pub enum MachineExpr {
    Immediate(x86_64::Immediate, x86_64::BitMode),
    Register(x86_64::Register, x86_64::BitMode),
}

pub fn bitmode_from_accsmode(accsmode: AccessMode) -> BitMode {
    match accsmode {
        AccessMode::BI => panic!("Cannot access individual bits"),
        AccessMode::QI => BitMode::Bit8,
        AccessMode::HI => BitMode::Bit16,
        AccessMode::SI => BitMode::Bit32,
        AccessMode::PSI => BitMode::Bit32,
        AccessMode::DI => BitMode::Bit64,
    }
}

pub fn mcexpr_from_regexpr(regexpr: &rtl::RegisterExpr) -> MachineExpr {
    MachineExpr::Register(
        regexpr
            .reg()
            .x86_64_machine_register()
            .expect("registers must be reloaded as x86-64"),
        bitmode_from_accsmode(regexpr.mode()),
    )
}

pub fn mcexpr_from_immexpr(immexpr: &rtl::ImmediateExpr) -> MachineExpr {
    MachineExpr::Immediate(
        match immexpr {
            rtl::ImmediateExpr::Int32(val) => x86_64::Immediate::Int32(*val as u32),
            rtl::ImmediateExpr::UInt32(val) => x86_64::Immediate::Int32(*val),
        },
        bitmode_from_accsmode(immexpr.as_access_mode()),
    )
}

pub fn mcexpr_from_destination(destexpr: &rtl::DestinationExpr) -> MachineExpr {
    match destexpr {
        rtl::DestinationExpr::Memory(_mem) => {
            panic!("memory as a destination is a work-in-progress")
        }
        rtl::DestinationExpr::Register(reg) => mcexpr_from_regexpr(reg),
    }
}

pub fn codegen_instruction_transfer(asm: &mut Vec<x86_64::Instruction>, ins: rtl::Transfer) {
    let mcexpr_destination = mcexpr_from_destination(ins.destination());
    let mcexpr_source = match ins.source() {
        rtl::Rtx::Destination(source) => mcexpr_from_destination(source),
        rtl::Rtx::Immediate(imm) => mcexpr_from_immexpr(imm),
    };

    let x86_64_ins = match (mcexpr_destination, mcexpr_source) {
        (MachineExpr::Immediate(..), _) => unreachable!(),
        (
            MachineExpr::Register(dest_reg, dest_mode),
            MachineExpr::Immediate(source_imm, source_mode),
        ) => {
            assert_eq!(dest_mode, source_mode);
            x86_64::mov_reg_imm(dest_mode, dest_reg, source_imm)
        }
        (
            MachineExpr::Register(dest_reg, dest_mode),
            MachineExpr::Register(source_reg, source_mode),
        ) => {
            assert_eq!(dest_mode, source_mode);
            x86_64::mov_reg_reg(dest_mode, dest_reg, source_reg)
        }
    };

    asm.push(x86_64_ins);
}

pub fn codegen_instruction_return(asm: &mut Vec<x86_64::Instruction>, _ret: rtl::Return) {
    asm.push(x86_64::retn(BitMode::Bit64));
}

pub fn codegen_instruction(asm: &mut Vec<x86_64::Instruction>, ins: rtl::Instruction) {
    match ins {
        rtl::Instruction::Transfer(transfer) => codegen_instruction_transfer(asm, transfer),
        rtl::Instruction::Return(ret) => codegen_instruction_return(asm, ret),
    }
}

#[cfg(test)]
mod tests {
    use crate::codegen_instruction;

    #[test]
    fn codegen_function_that_returns_10() {
        use gcf_rtl::prelude as rtl;
        use gcf_x86_64 as x86_64;

        use gcf_jit::os::JITHandle;
        use x86_64::EncodeInto;

        // We're using rax but moving values into it as it were eax.
        let reg = rtl::Register::preg(
            0,
            rtl::MachineRegister::x86_64(x86_64::Register::al),
            rtl::AccessMode::DI,
        );
        let rtl = [
            rtl::Instruction::Transfer(rtl::Transfer::new(
                rtl::DestinationExpr::Register(rtl::RegisterExpr::new(reg, rtl::AccessMode::SI)),
                rtl::Rtx::Immediate(rtl::ImmediateExpr::UInt32(10)),
            )),
            rtl::Instruction::Return(rtl::Return),
        ];
        let mut asm = Vec::new();
        for ins in rtl {
            codegen_instruction(&mut asm, ins);
        }
        let mut machine_code = Vec::new();
        for mc_ins in asm {
            mc_ins.encode_into(&mut machine_code).unwrap();
        }
        let jit_handle =
            match JITHandle::new(machine_code.as_slice(), x86_64::OpCode::nop.encoding().0) {
                Ok(x) => x,
                Err(errno) => panic!("Failed to allocatate executable memory: {errno}"),
            };
        type FunctionTy = unsafe extern "C" fn() -> u32;
        let func = unsafe {
            std::mem::transmute::<*const u8, FunctionTy>(jit_handle.as_ptr() as *const _)
        };
        assert_eq!(unsafe { func() }, 10);
    }
}
