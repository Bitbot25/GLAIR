/// Returns the size of a page. (Specifically on linux)
#[inline]
#[cfg(unix)]
pub fn page_size() -> usize {
    unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize }
}

#[cfg(windows)]
pub fn page_size() -> usize {
    compile_error!("Windows is currently unsupported.")
}

fn round_to_pow2(mut n: usize) -> usize {
    n -= 1;
    n |= n >> 1;
    n |= n >> 2;
    n |= n >> 4;
    n |= n >> 8;
    n |= n >> 16;
    n |= n >> 32;
    n + 1
}

#[inline(always)]
fn get_errno() -> i32 {
    unsafe {
        return *libc::__errno_location();
    }
}

pub struct JITHandle {
    ptr: *mut libc::c_void,
    length: usize,
}

impl JITHandle {
    #[cfg(unix)]
    pub fn new(data: &[u8], tmp_noop: u8) -> Result<Self, i32> {
        let page_sz = page_size();
        let segments = round_to_pow2((data.len() + page_sz - 1) / page_sz);
        let length = segments * page_sz;
        let ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                length,
                libc::PROT_WRITE,
                libc::MAP_ANONYMOUS | libc::MAP_PRIVATE,
                -1,
                0,
            )
        };
        if ptr as isize == -1 {
            return Err(get_errno());
        }
        dbg!(ptr);
        // Write noop instructions to the memory
        unsafe { std::ptr::write_bytes::<u8>(ptr as *mut _, tmp_noop, length) };
        eprintln!("write nops");
        // Write actual instructions
        unsafe { std::ptr::copy_nonoverlapping(data.as_ptr(), ptr as *mut _, data.len()) };
        eprintln!("wrote instructions");
        // Change protection to execute and read
        if unsafe { libc::mprotect(ptr, length, libc::PROT_EXEC | libc::PROT_READ) } != 0 {
            let errno = get_errno();
            // Unmap the page we just created.
            let _ = unsafe { libc::munmap(ptr, length) };
            return Err(errno);
        }
        eprintln!("changed protection");
        Ok(JITHandle { ptr, length })
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut libc::c_void {
        self.ptr
    }

    #[cfg(windows)]
    pub fn new(data: &[u8], tmp_noop: u8) -> Result<Self, i32> {
        compile_error!("Windows is currently unsupported")
    }

    /// Drops this JITHandle manually, returning the errno if it fails.
    #[cfg(unix)]
    #[must_use]
    #[inline(always)]
    pub fn destroy(self) -> Result<(), i32> {
        let res = unsafe { libc::munmap(self.ptr, self.length) };
        std::mem::forget(self);
        if res != 0 {
            Err(get_errno())
        } else {
            Ok(())
        }
    }

    #[cfg(windows)]
    #[must_use]
    pub fn destroy(self) -> Result<(), i32> {
        compile_error!("Windows is currently unsupported.")
    }
}

impl Drop for JITHandle {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.ptr, self.length);
        }
    }
}
