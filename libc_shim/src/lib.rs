#![no_std]
#![feature(core_intrinsics)]
#![feature(collections)]
#![feature(asm)]

extern crate libc;
#[macro_use]
extern crate sc;

extern crate collections;

use core::intrinsics;
use core::ptr;
use core::mem;
use collections::slice;

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
    /*
    if (off & OFF_MASK) {
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

pub fn strlen(s: *const libc::c_char) -> libc::size_t {
    // TODO: convert to checking word-size chunks
    // TODO: even better use arch-specific asm
    unsafe {
        for i in 0.. {
            if *s.offset(i) == 0 {
                return i as libc::size_t;
            }
        }
        intrinsics::unreachable();
    }
}

pub unsafe fn sigaddset(set: *mut libc::sigset_t,
                        signum: libc::c_int)
                        -> libc::c_int {
    let raw = slice::from_raw_parts_mut(set as *mut u8,
                                        mem::size_of::<libc::sigset_t>());
    let bit = (signum - 1) as usize;
    raw[bit / 8] |= 1 << (bit % 8);
    return 0;
}

pub fn sigaction(sig: libc::c_int,
                 sa: *const libc::sigaction,
                 old: *mut libc::sigaction)
                 -> libc::c_int {
    // check that signal number below _NSIG. _NSIG is 64
    // except mips where it is defined as 128

    unsafe {
        let mask_size = mem::size_of_val(&(*sa).sa_mask);
        // TOTHINK: we want to convert to signed type to check wether it is negative
        // make sure that syscall returning type is correct one
        // or maybe we should compare return value, errors are in [MAX-4096 .. MAX] range
        let ret = syscall!(RT_SIGACTION, sig, sa, old, mask_size) as
                  libc::c_int;
        if ret < 0 {
            return -1;
        }
    }
    return 0;
}

pub fn signal(sig: libc::c_int,
              handler: libc::sighandler_t)
              -> libc::sighandler_t {
    let mut sa_old: libc::sigaction = unsafe { mem::uninitialized() };
    // TODO: maybe instead of using zeroed here we should introduce a function similar to libc::sigemptyset?
    let set: libc::sigset_t = unsafe { mem::zeroed() };
    let sa = libc::sigaction {
        sa_sigaction: handler,
        sa_flags: libc::SA_RESTART,
        sa_mask: set,
        _restorer: ptr::null_mut(),
    };
    if sigaction(sig, &sa, &mut sa_old) < 0 {
        return libc::SIG_ERR;
    }
    return sa_old.sa_sigaction;
}
