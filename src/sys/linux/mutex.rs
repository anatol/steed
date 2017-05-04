// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use linux;
use cell::UnsafeCell;
use ptr;
use sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering};
use thread;
use usize;

pub struct Mutex {
    // TOTHINK: as locked is only 1 bit we can save some space by
    // using part of 'waiters' field to store 'locked' bit. This way
    // we don't need 'locked' field and save some memory.
    locked: UnsafeCell<AtomicBool>,
    // Number of threads blocked in kernel
    waiters: AtomicUsize,
}

unsafe impl Send for Mutex {}
unsafe impl Sync for Mutex {}

impl Mutex {
    pub const fn new() -> Mutex {
        Mutex {
            locked: UnsafeCell::new(AtomicBool::new(false)),
            waiters: AtomicUsize::new(0),
        }
    }
    #[inline]
    pub unsafe fn locked_raw(&self) -> &mut AtomicBool {
        &mut *self.locked.get()
    }
    #[inline]
    pub unsafe fn init(&self) {
    }
    #[inline]
    pub unsafe fn try_lock(&self) -> bool {
        let locked_prev = self.locked_raw().compare_and_swap(false, true, Ordering::Acquire);
        !locked_prev
    }
    #[inline]
    pub unsafe fn lock(&self) {
        loop {
            let locked_prev = self.locked_raw().compare_and_swap(false, true, Ordering::Acquire);
            if !locked_prev {
                break;
            }

            self.waiters.fetch_add(1, Ordering::Relaxed);
            let futex: *const bool = self.locked_raw().get_mut();
            syscall!(FUTEX, futex, linux::FUTEX_WAIT_PRIVATE, 0, 0, 0, 0);
            self.waiters.fetch_sub(1, Ordering::Relaxed);
        }
    }
    #[inline]
    pub unsafe fn unlock(&self) {
        let locked_prev = self.locked_raw().compare_and_swap(true, false, Ordering::Release);
        if locked_prev {
            if self.waiters.load(Ordering::Relaxed) != 0 {
                // As an optimization we can do some small amount of spins and check if the lock gets
                // unlocked. And only if spin does not work then go to sleep.

                let futex: *const bool = self.locked_raw().get_mut();
                syscall!(FUTEX, futex, linux::FUTEX_WAKE_PRIVATE, 1, 0, 0, 0);
            }
        } else {
            panic!("mutex is not locked");
        }
    }
    #[inline]
    pub unsafe fn destroy(&self) {
    }
}

pub struct ReentrantMutex {
    locked: UnsafeCell<AtomicBool>,
    waiters: AtomicUsize,
    owner: AtomicPtr<libc::pthread_t>,
    // Number of times we reentered to this lock, the counter starts from 0.
    // To help reduce memory traffic for common case (mutex is taken only once) this couter
    // start with 0. If the mutex is locked and reentrance_count is zero then it means
    // current thread locked it only once.
    reentrance_count: usize,
}

unsafe impl Send for ReentrantMutex {}
unsafe impl Sync for ReentrantMutex {}

impl ReentrantMutex {
    pub const fn new() -> ReentrantMutex {
        ReentrantMutex {
            locked: UnsafeCell::new(AtomicBool::new(false)),
            waiters: AtomicUsize::new(0),
            owner: AtomicPtr::new(ptr::null_mut()),
            reentrance_count: 0,
        }
    }
    #[inline]
    pub unsafe fn locked_raw(&self) -> &mut AtomicBool {
        &mut *self.locked.get()
    }
    #[inline]
    pub unsafe fn init(&self) {
    }
    #[inline]
    pub unsafe fn try_lock(&self) -> bool {
        let locked_prev = self.locked_raw().compare_and_swap(false, true, Ordering::Acquire);
        if !locked_prev {
            return true
        } else {
            // check who owns the lock, is it us?
            if self.owner.load(Ordering::Relaxed) == thread::current().thread {
                self.reentrance_count += 1;
                return true;
            }
            return false;
        }
    }
    #[inline]
    pub unsafe fn lock(&self) {
        loop {
            let locked_prev = self.locked_raw().compare_and_swap(false, true, Ordering::Acquire);
            if !locked_prev {
                self.owner.store(thread::current().thread, Ordering::Relaxed);
                break;
            } else if self.owner.load(Ordering::Relaxed) == thread::current().thread {
                self.reentrance_count += 1;
                break;
            }

            self.waiters.fetch_add(1, Ordering::Relaxed);
            let futex: *const bool = self.locked_raw().get_mut();
            syscall!(FUTEX, futex, linux::FUTEX_WAIT_PRIVATE, 0, 0, 0, 0);
            self.waiters.fetch_sub(1, Ordering::Relaxed);
        }
    }
    #[inline]
    pub unsafe fn unlock(&self) {
        if self.owner.load(Ordering::Relaxed) != thread::current() {
            panic!("unlocking mutex owned by other thread");
        }

        if self.reentrance_count == 0 {
            // common case, we are the only lock in this thread
            self.owner.store(ptr::null_mut(), Ordering::Relaxed);

            let locked_prev = self.locked_raw().compare_and_swap(true, false, Ordering::Release);
            if locked_prev {
                if self.waiters.load(Ordering::Relaxed) != 0 {
                    // As an optimization we can do some small amount of spins and check if the lock gets
                    // unlocked. And only if spin does not work then go to sleep.

                    let futex: *const bool = self.locked_raw().get_mut();
                    syscall!(FUTEX, futex, linux::FUTEX_WAKE_PRIVATE, 1, 0, 0, 0);
                }
            } else {
                panic!("mutex is not locked");
            }
        } else {
            self.reentrance_count -= 1;
        }
    }
}
