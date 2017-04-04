#![no_std]
#![feature(core_intrinsics)]
#![feature(asm)]

extern crate libc;
#[macro_use]
extern crate sc;

use core::intrinsics;

pub fn exit_group(code: libc::c_int) -> ! {
    unsafe {
        syscall!(EXIT_GROUP, code);
        intrinsics::unreachable()
    }
}

pub fn abort() -> ! {
    unsafe {
        let tid = syscall!(GETTID);
        syscall!(TKILL, tid, libc::SIGABRT);
        asm!("hlt");
        syscall!(TKILL, tid, libc::SIGKILL);
    }
    exit_group(127)
}

pub fn mmap(addr: *mut libc::c_void,
            len: libc::size_t,
            prot: libc::c_int,
            flags: libc::c_int,
            fd: libc::c_int,
            offset: libc::off_t)
            -> *mut libc::c_void {

    /*if (off & OFF_MASK) {
        errno = EINVAL;
        return MAP_FAILED;
    }
    if (len >= PTRDIFF_MAX) {
        errno = ENOMEM;
        return MAP_FAILED;
    }
    if (flags & MAP_FIXED) {
        __vm_wait();
    }
*/
    unsafe {
        // on 32 bit platform it is possible to use MMAP2
        syscall!(MMAP, addr, len, prot, flags, fd, offset) as *mut libc::c_void
    }
}

pub fn munmap(addr: *mut libc::c_void, len: libc::size_t) -> libc::c_int {
    unsafe { syscall!(MUNMAP, addr, len) as libc::c_int }
}

pub fn write(fd: libc::c_int,
             buf: *const libc::c_void,
             count: libc::size_t)
             -> libc::ssize_t {
    unsafe { syscall!(WRITE, fd, buf, count) as libc::ssize_t }
}
