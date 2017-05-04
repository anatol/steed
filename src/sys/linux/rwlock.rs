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
use sync::atomic::{AtomicUsize, Ordering};
use {isize, usize};

// value that signals rwlock is locked with a writer
const RWLOCK_WRITER: usize = usize::MAX;

pub struct RWLock {
    // Number of users for this rwlock
    // Zero means no users
    // Value equal to `RWLOCK_WRITER` means it is locked by a writer
    // Any other value - number of readears currently holding the lock
    users: UnsafeCell<AtomicUsize>,

    // Number of blocked threads that wait when the lock becomes available
    waiters: AtomicUsize,
}

unsafe impl Send for RWLock {}
unsafe impl Sync for RWLock {}

impl RWLock {
    pub const fn new() -> RWLock {
        RWLock {
            // We use UnsafeCell because we need address of the pointer for futex() syscall
            users: UnsafeCell::new(AtomicUsize::new(0)),
            waiters: AtomicUsize::new(0),
        }
    }
    #[inline]
    pub unsafe fn users_raw(&self) -> &mut AtomicUsize {
        &mut *self.users.get()
    }
    #[inline]
    pub unsafe fn read(&self) {
        let mut users = self.users_raw().load(Ordering::Acquire);

        loop {
            if users == RWLOCK_WRITER {
                self.waiters.fetch_add(1, Ordering::Relaxed);
                let futex: *const usize = self.users_raw().get_mut();
                syscall!(FUTEX, futex, linux::FUTEX_WAIT_PRIVATE, users, 0, 0, 0);
                self.waiters.fetch_sub(1, Ordering::Relaxed);

                users = self.users_raw().load(Ordering::Acquire);
            } else if users == RWLOCK_WRITER - 1 {
                panic!("rwlock maximum reader count exceeded");
            } else {
                let users_prev = self.users_raw().compare_and_swap(users, users + 1, Ordering::Acquire);
                if users == users_prev {
                    // atomic swap was successfull, we are good
                    break;
                }
                users = users_prev;
            }
        }
    }
    #[inline]
    pub unsafe fn try_read(&self) -> bool {
        let mut users = self.users_raw().load(Ordering::Acquire);

        loop {
            if users == RWLOCK_WRITER {
                return false;
            } else if users == RWLOCK_WRITER - 1 {
                panic!("rwlock maximum reader count exceeded");
            } else {
                let users_prev = self.users_raw().compare_and_swap(users, users + 1, Ordering::Acquire);
                if users == users_prev {
                    // atomic swap was successfull, we are good
                    return true;
                }
                users = users_prev;
            }
        }
    }
    #[inline]
    pub unsafe fn write(&self) {
        loop {
            let users_prev = self.users_raw().compare_and_swap(0, RWLOCK_WRITER, Ordering::Acquire);
            if users_prev == 0 {
                break;
            }

            self.waiters.fetch_add(1, Ordering::Relaxed);
            let futex: *const usize = self.users_raw().get_mut();
            syscall!(FUTEX, futex, linux::FUTEX_WAIT_PRIVATE, users_prev, 0, 0, 0);
            self.waiters.fetch_sub(1, Ordering::Relaxed);
        }
    }
    #[inline]
    pub unsafe fn try_write(&self) -> bool {
        let users_prev = self.users_raw().compare_and_swap(0, RWLOCK_WRITER, Ordering::Acquire);
        users_prev == 0
    }
    #[inline]
    pub unsafe fn read_unlock(&self) {
        let mut users = self.users_raw().load(Ordering::Release);

        loop {
            if users == RWLOCK_WRITER {
                panic!("rwlock is locked by a writer");
            }
            if users == 0 {
                panic!("rwlock is not locked by a reader");
            }
            let users_prev = self.users_raw().compare_and_swap(users, users - 1, Ordering::Release);
            if users == users_prev {
                if users - 1 == 0 && self.waiters.load(Ordering::Relaxed) != 0 {
                    // As an optimization we can do some small amount of spins and check if the lock gets
                    // unlocked. And only if spin does not work then go to sleep.

                    let futex: *const usize = self.users_raw().get_mut();
                    // We just dropped a read lock. If we had waiters then they all must be writers
                    // (readers would not block). In this case no need to wake more than 1 waiter.
                    syscall!(FUTEX, futex, linux::FUTEX_WAKE_PRIVATE, 1, 0, 0, 0);
                }
                break;
            }
            users = users_prev;
        }
    }
    #[inline]
    pub unsafe fn write_unlock(&self) {
        let users_prev = self.users_raw().compare_and_swap(RWLOCK_WRITER, 0, Ordering::Release);
        if users_prev == RWLOCK_WRITER {
            if self.waiters.load(Ordering::Relaxed) != 0 {
                // As an optimization we can do some small amount of spins and check if the lock gets
                // unlocked. And only if spin does not work then go to sleep.

                let futex: *const usize = self.users_raw().get_mut();
                // There can be both reader and writer waiters. Wake all of them and let's the fight
                // begin.
                syscall!(FUTEX, futex, linux::FUTEX_WAKE_PRIVATE, isize::MAX, 0, 0, 0);
            }
        } else {
            panic!("rwlock is not locked by a writer");
        }
    }
    #[inline]
    pub unsafe fn destroy(&self) {
    }
}
