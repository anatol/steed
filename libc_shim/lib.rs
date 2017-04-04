#![no_std]

#[inline(always)]
pub fn exit_group(code: c_int) -> ! {
    unsafe {
        syscall!(EXIT_GROUP, code);
        intrinsics::unreachable()
    }
}

pub fn abort() -> ! {
    unsafe {
        let tid = syscall(GETTID);
        syscall!(TKILL, tid, SIGABRT);
        asm!("hlt");
        syscall!(TKILL, tid, SIGKILL);
    }
    exit_group(127)
}
