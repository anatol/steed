use ctypes::c_char;
use linux;

pub fn errno() -> i32 {
    panic!("no C-compatible errno variable");
}

pub fn error_string(errno: i32) -> String {
    linux::errno::error_string(errno).map(|s| s.into()).unwrap_or_else(|| {
        format!("Unknown OS error ({})", errno)
    })
}

pub fn exit(code: i32) -> ! {
    unsafe { linux::exit_group(code) }
}

pub fn page_size() -> usize {
    unimplemented!();
}

// TODO(steed): Fix this unsafety, should be *const c_char elements.
static ENVIRON: [usize; 1] = [0];

pub unsafe fn environ() -> *const *const c_char {
    let env: *const usize = ENVIRON.as_ptr();
    env as *const *const c_char
}
