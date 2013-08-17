// Copyright 2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use container::Container;
use from_str::FromStr;
use libc;
use option::{Some, None};
use os;
use str::StrSlice;
use unstable::atomics::{AtomicInt, INIT_ATOMIC_INT, SeqCst};

#[cfg(target_os="macos")]
use unstable::running_on_valgrind;

// Indicates whether we should perform expensive sanity checks, including rtassert!
pub static ENFORCE_SANITY: bool = !cfg!(rtopt) || cfg!(rtdebug) || cfg!(rtassert);

/// Get the number of cores available
pub fn num_cpus() -> uint {
    #[fixed_stack_segment]; #[inline(never)];

    unsafe {
        return rust_get_num_cpus();
    }

    extern {
        fn rust_get_num_cpus() -> libc::uintptr_t;
    }
}

/// Valgrind has a fixed-sized array (size around 2000) of segment descriptors wired into it; this
/// is a hard limit and requires rebuilding valgrind if you want to go beyond it. Normally this is
/// not a problem, but in some tests, we produce a lot of threads casually. Making lots of threads
/// alone might not be a problem _either_, except on OSX, the segments produced for new threads
/// _take a while_ to get reclaimed by the OS. Combined with the fact that libuv schedulers fork off
/// a separate thread for polling fsevents on OSX, we get a perfect storm of creating "too many
/// mappings" for valgrind to handle when running certain stress tests in the runtime.
#[cfg(target_os="macos")]
pub fn limit_thread_creation_due_to_osx_and_valgrind() -> bool {
    running_on_valgrind()
}

#[cfg(not(target_os="macos"))]
pub fn limit_thread_creation_due_to_osx_and_valgrind() -> bool {
    false
}

/// Get's the number of scheduler threads requested by the environment
/// either `RUST_THREADS` or `num_cpus`.
pub fn default_sched_threads() -> uint {
    match os::getenv("RUST_THREADS") {
        Some(nstr) => FromStr::from_str(nstr).unwrap(),
        None => {
            if limit_thread_creation_due_to_osx_and_valgrind() {
                1
            } else {
                num_cpus()
            }
        }
    }
}

pub fn dumb_println(s: &str) {
    use io::WriterUtil;
    let dbg = ::libc::STDERR_FILENO as ::io::fd_t;
    dbg.write_str(s + "\n");
}

pub fn abort(msg: &str) -> ! {
    let msg = if !msg.is_empty() { msg } else { "aborted" };
    let hash = msg.iter().fold(0, |accum, val| accum + (val as uint) );
    let quote = match hash % 10 {
        0 => "
It was from the artists and poets that the pertinent answers came, and I
know that panic would have broken loose had they been able to compare notes.
As it was, lacking their original letters, I half suspected the compiler of
having asked leading questions, or of having edited the correspondence in
corroboration of what he had latently resolved to see.",
        1 => "
There are not many persons who know what wonders are opened to them in the
stories and visions of their youth; for when as children we listen and dream,
we think but half-formed thoughts, and when as men we try to remember, we are
dulled and prosaic with the poison of life. But some of us awake in the night
with strange phantasms of enchanted hills and gardens, of fountains that sing
in the sun, of golden cliffs overhanging murmuring seas, of plains that stretch
down to sleeping cities of bronze and stone, and of shadowy companies of heroes
that ride caparisoned white horses along the edges of thick forests; and then
we know that we have looked back through the ivory gates into that world of
wonder which was ours before we were wise and unhappy.",
        2 => "
Instead of the poems I had hoped for, there came only a shuddering blackness
and ineffable loneliness; and I saw at last a fearful truth which no one had
ever dared to breathe before — the unwhisperable secret of secrets — The fact
that this city of stone and stridor is not a sentient perpetuation of Old New
York as London is of Old London and Paris of Old Paris, but that it is in fact
quite dead, its sprawling body imperfectly embalmed and infested with queer
animate things which have nothing to do with it as it was in life.",
        3 => "
The ocean ate the last of the land and poured into the smoking gulf, thereby
giving up all it had ever conquered. From the new-flooded lands it flowed
again, uncovering death and decay; and from its ancient and immemorial bed it
trickled loathsomely, uncovering nighted secrets of the years when Time was
young and the gods unborn. Above the waves rose weedy remembered spires. The
moon laid pale lilies of light on dead London, and Paris stood up from its damp
grave to be sanctified with star-dust. Then rose spires and monoliths that were
weedy but not remembered; terrible spires and monoliths of lands that men never
knew were lands...",
        4 => "
There was a night when winds from unknown spaces whirled us irresistibly into
limitless vacum beyond all thought and entity. Perceptions of the most
maddeningly untransmissible sort thronged upon us; perceptions of infinity
which at the time convulsed us with joy, yet which are now partly lost to my
memory and partly incapable of presentation to others.",
        _ => "You've met with a terrible fate, haven't you?"
    };
    rterrln!("%s", "");
    rterrln!("%s", quote);
    rterrln!("%s", "");
    rterrln!("fatal runtime error: %s", msg);

    abort();

    fn abort() -> ! {
        #[fixed_stack_segment]; #[inline(never)];
        unsafe { libc::abort() }
    }
}

static mut EXIT_STATUS: AtomicInt = INIT_ATOMIC_INT;

pub fn set_exit_status(code: int) {
    unsafe { EXIT_STATUS.store(code, SeqCst) }
}

pub fn get_exit_status() -> int {
    unsafe { EXIT_STATUS.load(SeqCst) }
}
