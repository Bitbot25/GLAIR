pub mod os;

#[cfg(test)]
mod tests {
    use gcf_x86_64::{BitMode, EncodeInto, Immediate, OpCode, Register};

    use crate::os::JITHandle;

    #[test]
    fn function_returns_10() {
        type Function = unsafe extern "C" fn() -> u64;

        let mut buf = Vec::new();
        let instructions = [
            gcf_x86_64::mov_reg_imm(BitMode::Bit64, Register::al, Immediate::Int64(10)),
            gcf_x86_64::retn(BitMode::Bit64),
        ];

        for instruction in instructions {
            instruction.encode_into(&mut buf).unwrap();
        }

        let handle = match JITHandle::new(buf.as_slice(), OpCode::nop.encoding().0) {
            Ok(handle) => handle,
            Err(e) => panic!("Failed to create executable memory: {e:?}"),
        };

        let func = unsafe { std::mem::transmute::<*const libc::c_void, Function>(handle.as_ptr()) };
        let result = unsafe { func() };
        assert_eq!(result, 10);
    }
}
