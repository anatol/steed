// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation of `std::os` functionality for unix systems

#![allow(unused_imports)] // lots of cfg code here

use os::unix::prelude::*;

use core::intrinsics;
use error::Error as StdError;
use ffi::{CString, CStr, OsString, OsStr};
use fmt;
use io;
use iter;
use libc::{self, c_int, c_char, c_void};
use marker::PhantomData;
use mem;
use memchr;
use path::{self, PathBuf};
use ptr;
use slice;
use str;
use sys_common::mutex::Mutex;
use sys::cvt;
use sys::fd;
use vec;

static ENV_LOCK: Mutex = Mutex::new();


/// Returns the platform-specific value of errno
#[cfg(not(target_os = "dragonfly"))]
pub fn errno() -> i32 {
    // Do not use errno. Remove this function and use exit code directly from the syscall
    0
}

/// Sets the platform-specific value of errno
#[cfg(any(target_os = "solaris", target_os = "fuchsia"))] // only needed for readdir so far
pub fn set_errno(e: i32) {
    unsafe { *errno_location() = e as c_int }
}

#[cfg(target_os = "dragonfly")]
pub fn errno() -> i32 {
    extern "C" {
        #[thread_local]
        static errno: c_int;
    }

    unsafe { errno as i32 }
}

/// Gets a detailed string description for the given error number.
pub fn error_string(errno: i32) -> String {
    // Some errno values coincide on some platforms, while they don't on others.
    #[allow(unreachable_patterns)]
    match errno {
            libc::EPERM => "Operation not permitted",
            libc::ENOENT => "No such file or directory",
            libc::ESRCH => "No such process",
            libc::EINTR => "Interrupted system call",
            libc::EIO => "I/O error",
            libc::ENXIO => "No such device or address",
            libc::E2BIG => "Argument list too long",
            libc::ENOEXEC => "Exec format error",
            libc::EBADF => "Bad file number",
            libc::ECHILD => "No child processes",
            libc::EAGAIN => "Try again",
            libc::ENOMEM => "Out of memory",
            libc::EACCES => "Permission denied",
            libc::EFAULT => "Bad address",
            libc::ENOTBLK => "Block device required",
            libc::EBUSY => "Device or resource busy",
            libc::EEXIST => "File exists",
            libc::EXDEV => "Cross-device link",
            libc::ENODEV => "No such device",
            libc::ENOTDIR => "Not a directory",
            libc::EISDIR => "Is a directory",
            libc::EINVAL => "Invalid argument",
            libc::ENFILE => "File table overflow",
            libc::EMFILE => "Too many open files",
            libc::ENOTTY => "Not a typewriter",
            libc::ETXTBSY => "Text file busy",
            libc::EFBIG => "File too large",
            libc::ENOSPC => "No space left on device",
            libc::ESPIPE => "Illegal seek",
            libc::EROFS => "Read-only file system",
            libc::EMLINK => "Too many links",
            libc::EPIPE => "Broken pipe",
            libc::EDOM => "Math argument out of domain of func",
            libc::ERANGE => "Math result not representable",
            libc::EDEADLK => "Resource deadlock would occur",
            libc::ENAMETOOLONG => "File name too long",
            libc::ENOLCK => "No record locks available",
            libc::ENOSYS => "Invalid system call number",
            libc::ENOTEMPTY => "Directory not empty",
            libc::ELOOP => "Too many symbolic links encountered",
            libc::EWOULDBLOCK => "Operation would block",
            libc::ENOMSG => "No message of desired type",
            libc::EIDRM => "Identifier removed",
            libc::ECHRNG => "Channel number out of range",
            libc::EL2NSYNC => "Level 2 not synchronized",
            libc::EL3HLT => "Level 3 halted",
            libc::EL3RST => "Level 3 reset",
            libc::ELNRNG => "Link number out of range",
            libc::EUNATCH => "Protocol driver not attached",
            libc::ENOCSI => "No CSI structure available",
            libc::EL2HLT => "Level 2 halted",
            libc::EBADE => "Invalid exchange",
            libc::EBADR => "Invalid request descriptor",
            libc::EXFULL => "Exchange full",
            libc::ENOANO => "No anode",
            libc::EBADRQC => "Invalid request code",
            libc::EBADSLT => "Invalid slot",
            libc::EDEADLOCK => "Resource deadlock would occur",
            libc::EBFONT => "Bad font file format",
            libc::ENOSTR => "Device not a stream",
            libc::ENODATA => "No data available",
            libc::ETIME => "Timer expired",
            libc::ENOSR => "Out of streams resources",
            libc::ENONET => "Machine is not on the network",
            libc::ENOPKG => "Package not installed",
            libc::EREMOTE => "Object is remote",
            libc::ENOLINK => "Link has been severed",
            libc::EADV => "Advertise error",
            libc::ESRMNT => "Srmount error",
            libc::ECOMM => "Communication error on send",
            libc::EPROTO => "Protocol error",
            libc::EMULTIHOP => "Multihop attempted",
            libc::EDOTDOT => "RFS specific error",
            libc::EBADMSG => "Not a data message",
            libc::EOVERFLOW => "Value too large for defined data type",
            libc::ENOTUNIQ => "Name not unique on network",
            libc::EBADFD => "File descriptor in bad state",
            libc::EREMCHG => "Remote address changed",
            libc::ELIBACC => "Can not access a needed shared library",
            libc::ELIBBAD => "Accessing a corrupted shared library",
            libc::ELIBSCN => ".lib section in a.out corrupted",
            libc::ELIBMAX => "Attempting to link in too many shared libraries",
            libc::ELIBEXEC => "Cannot exec a shared library directly",
            libc::EILSEQ => "Illegal byte sequence",
            libc::ERESTART => "Interrupted system call should be restarted",
            libc::ESTRPIPE => "Streams pipe error",
            libc::EUSERS => "Too many users",
            libc::ENOTSOCK => "Socket operation on non-socket",
            libc::EDESTADDRREQ => "Destination address required",
            libc::EMSGSIZE => "Message too long",
            libc::EPROTOTYPE => "Protocol wrong type for socket",
            libc::ENOPROTOOPT => "Protocol not available",
            libc::EPROTONOSUPPORT => "Protocol not supported",
            libc::ESOCKTNOSUPPORT => "Socket type not supported",
            libc::EOPNOTSUPP => "Operation not supported on transport endpoint",
            libc::EPFNOSUPPORT => "Protocol family not supported",
            libc::EAFNOSUPPORT => "Address family not supported by protocol",
            libc::EADDRINUSE => "Address already in use",
            libc::EADDRNOTAVAIL => "Cannot assign requested address",
            libc::ENETDOWN => "Network is down",
            libc::ENETUNREACH => "Network is unreachable",
            libc::ENETRESET => "Network dropped connection because of reset",
            libc::ECONNABORTED => "Software caused connection abort",
            libc::ECONNRESET => "Connection reset by peer",
            libc::ENOBUFS => "No buffer space available",
            libc::EISCONN => "Transport endpoint is already connected",
            libc::ENOTCONN => "Transport endpoint is not connected",
            libc::ESHUTDOWN => "Cannot send after transport endpoint shutdown",
            libc::ETOOMANYREFS => "Too many references: cannot splice",
            libc::ETIMEDOUT => "Connection timed out",
            libc::ECONNREFUSED => "Connection refused",
            libc::EHOSTDOWN => "Host is down",
            libc::EHOSTUNREACH => "No route to host",
            libc::EALREADY => "Operation already in progress",
            libc::EINPROGRESS => "Operation now in progress",
            libc::ESTALE => "Stale file handle",
            libc::EUCLEAN => "Structure needs cleaning",
            libc::ENOTNAM => "Not a XENIX named type file",
            libc::ENAVAIL => "No XENIX semaphores available",
            libc::EISNAM => "Is a named type file",
            libc::EREMOTEIO => "Remote I/O error",
            libc::EDQUOT => "Quota exceeded",
            libc::ENOMEDIUM => "No medium found",
            libc::EMEDIUMTYPE => "Wrong medium type",
            libc::ECANCELED => "Operation Canceled",
            libc::ENOKEY => "Required key not available",
            libc::EKEYEXPIRED => "Key has expired",
            libc::EKEYREVOKED => "Key has been revoked",
            libc::EKEYREJECTED => "Key was rejected by service",
            libc::EOWNERDEAD => "Owner died",
            libc::ENOTRECOVERABLE => "State not recoverable",
            libc::ERFKILL => "Operation not possible due to RF-kill",
            libc::EHWPOISON => "Memory page has hardware error",
            _ => panic!("Unknown error code {}", errno),
        }
        .to_string()
}

pub fn getcwd() -> io::Result<PathBuf> {
    let mut buf = Vec::with_capacity(512);
    loop {
        unsafe {
            let ptr = buf.as_mut_ptr() as *mut libc::c_char;
            if !libc::getcwd(ptr, buf.capacity()).is_null() {
                let len = CStr::from_ptr(buf.as_ptr() as *const libc::c_char)
                    .to_bytes()
                    .len();
                buf.set_len(len);
                buf.shrink_to_fit();
                return Ok(PathBuf::from(OsString::from_vec(buf)));
            } else {
                let error = io::Error::last_os_error();
                if error.raw_os_error() != Some(libc::ERANGE) {
                    return Err(error);
                }
            }

            // Trigger the internal buffer resizing logic of `Vec` by requiring
            // more space than the current capacity.
            let cap = buf.capacity();
            buf.set_len(cap);
            buf.reserve(1);
        }
    }
}

pub fn chdir(p: &path::Path) -> io::Result<()> {
    let p: &OsStr = p.as_ref();
    let p = CString::new(p.as_bytes())?;
    unsafe {
        match libc::chdir(p.as_ptr()) == (0 as c_int) {
            true => Ok(()),
            false => Err(io::Error::last_os_error()),
        }
    }
}

pub struct SplitPaths<'a> {
    iter: iter::Map<slice::Split<'a, u8, fn(&u8) -> bool>,
                    fn(&'a [u8]) -> PathBuf>,
}

pub fn split_paths(unparsed: &OsStr) -> SplitPaths {
    fn bytes_to_path(b: &[u8]) -> PathBuf {
        PathBuf::from(<OsStr as OsStrExt>::from_bytes(b))
    }
    fn is_colon(b: &u8) -> bool {
        *b == b':'
    }
    let unparsed = unparsed.as_bytes();
    SplitPaths {
        iter: unparsed.split(is_colon as fn(&u8) -> bool)
                      .map(bytes_to_path as fn(&[u8]) -> PathBuf)
    }
}

impl<'a> Iterator for SplitPaths<'a> {
    type Item = PathBuf;
    fn next(&mut self) -> Option<PathBuf> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

#[derive(Debug)]
pub struct JoinPathsError;

pub fn join_paths<I, T>(paths: I) -> Result<OsString, JoinPathsError>
    where I: Iterator<Item = T>,
          T: AsRef<OsStr>
{
    let mut joined = Vec::new();
    let sep = b':';

    for (i, path) in paths.enumerate() {
        let path = path.as_ref().as_bytes();
        if i > 0 {
            joined.push(sep)
        }
        if path.contains(&sep) {
            return Err(JoinPathsError);
        }
        joined.extend_from_slice(path);
    }
    Ok(OsStringExt::from_vec(joined))
}

impl fmt::Display for JoinPathsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        "path segment contains separator `:`".fmt(f)
    }
}

impl StdError for JoinPathsError {
    fn description(&self) -> &str {
        "failed to join paths"
    }
}

#[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
pub fn current_exe() -> io::Result<PathBuf> {
    unsafe {
        let mut mib = [libc::CTL_KERN as c_int,
                       libc::KERN_PROC as c_int,
                       libc::KERN_PROC_PATHNAME as c_int,
                       -1 as c_int];
        let mut sz = 0;
        cvt(libc::sysctl(mib.as_mut_ptr(),
                         mib.len() as ::libc::c_uint,
                         ptr::null_mut(),
                         &mut sz,
                         ptr::null_mut(),
                         0))?;
        if sz == 0 {
            return Err(io::Error::last_os_error());
        }
        let mut v: Vec<u8> = Vec::with_capacity(sz);
        cvt(libc::sysctl(mib.as_mut_ptr(),
                         mib.len() as ::libc::c_uint,
                         v.as_mut_ptr() as *mut libc::c_void,
                         &mut sz,
                         ptr::null_mut(),
                         0))?;
        if sz == 0 {
            return Err(io::Error::last_os_error());
        }
        v.set_len(sz - 1); // chop off trailing NUL
        Ok(PathBuf::from(OsString::from_vec(v)))
    }
}

#[cfg(target_os = "netbsd")]
pub fn current_exe() -> io::Result<PathBuf> {
    ::fs::read_link("/proc/curproc/exe")
}

#[cfg(any(target_os = "bitrig", target_os = "openbsd"))]
pub fn current_exe() -> io::Result<PathBuf> {
    unsafe {
        let mut mib = [libc::CTL_KERN,
                       libc::KERN_PROC_ARGS,
                       libc::getpid(),
                       libc::KERN_PROC_ARGV];
        let mib = mib.as_mut_ptr();
        let mut argv_len = 0;
        cvt(libc::sysctl(mib,
                         4,
                         ptr::null_mut(),
                         &mut argv_len,
                         ptr::null_mut(),
                         0))?;
        let mut argv = Vec::<*const libc::c_char>::with_capacity(argv_len as
                                                                 usize);
        cvt(libc::sysctl(mib,
                         4,
                         argv.as_mut_ptr() as *mut _,
                         &mut argv_len,
                         ptr::null_mut(),
                         0))?;
        argv.set_len(argv_len as usize);
        if argv[0].is_null() {
            return Err(io::Error::new(io::ErrorKind::Other,
                                      "no current exe available"));
        }
        let argv0 = CStr::from_ptr(argv[0]).to_bytes();
        if argv0[0] == b'.' || argv0.iter().any(|b| *b == b'/') {
            ::fs::canonicalize(OsStr::from_bytes(argv0))
        } else {
            Ok(PathBuf::from(OsStr::from_bytes(argv0)))
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "android", target_os = "emscripten"))]
pub fn current_exe() -> io::Result<PathBuf> {
    ::fs::read_link("/proc/self/exe")
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub fn current_exe() -> io::Result<PathBuf> {
    extern "C" {
        fn _NSGetExecutablePath(buf: *mut libc::c_char,
                                bufsize: *mut u32)
                                -> libc::c_int;
    }
    unsafe {
        let mut sz: u32 = 0;
        _NSGetExecutablePath(ptr::null_mut(), &mut sz);
        if sz == 0 {
            return Err(io::Error::last_os_error());
        }
        let mut v: Vec<u8> = Vec::with_capacity(sz as usize);
        let err = _NSGetExecutablePath(v.as_mut_ptr() as *mut i8, &mut sz);
        if err != 0 {
            return Err(io::Error::last_os_error());
        }
        v.set_len(sz as usize - 1); // chop off trailing NUL
        Ok(PathBuf::from(OsString::from_vec(v)))
    }
}

#[cfg(any(target_os = "solaris"))]
pub fn current_exe() -> io::Result<PathBuf> {
    extern "C" {
        fn getexecname() -> *const c_char;
    }
    unsafe {
        let path = getexecname();
        if path.is_null() {
            Err(io::Error::last_os_error())
        } else {
            let filename = CStr::from_ptr(path).to_bytes();
            let path = PathBuf::from(<OsStr as OsStrExt>::from_bytes(filename));

            // Prepend a current working directory to the path if
            // it doesn't contain an absolute pathname.
            if filename[0] == b'/' {
                Ok(path)
            } else {
                getcwd().map(|cwd| cwd.join(path))
            }
        }
    }
}

#[cfg(target_os = "haiku")]
pub fn current_exe() -> io::Result<PathBuf> {
    // Use Haiku's image info functions
    #[repr(C)]
    struct image_info {
        id: i32,
        type_: i32,
        sequence: i32,
        init_order: i32,
        init_routine: *mut libc::c_void, // function pointer
        term_routine: *mut libc::c_void, // function pointer
        device: libc::dev_t,
        node: libc::ino_t,
        name: [libc::c_char; 1024], // MAXPATHLEN
        text: *mut libc::c_void,
        data: *mut libc::c_void,
        text_size: i32,
        data_size: i32,
        api_version: i32,
        abi: i32,
    }

    unsafe {
        extern "C" {
            fn _get_next_image_info(team_id: i32,
                                    cookie: *mut i32,
                                    info: *mut image_info,
                                    size: i32)
                                    -> i32;
        }

        let mut info: image_info = mem::zeroed();
        let mut cookie: i32 = 0;
        // the executable can be found at team id 0
        let result = _get_next_image_info(0,
                                          &mut cookie,
                                          &mut info,
                                          mem::size_of::<image_info>() as i32);
        if result != 0 {
            use io::ErrorKind;
            Err(io::Error::new(ErrorKind::Other,
                               "Error getting executable path"))
        } else {
            let name = CStr::from_ptr(info.name.as_ptr()).to_bytes();
            Ok(PathBuf::from(OsStr::from_bytes(name)))
        }
    }
}

#[cfg(target_os = "fuchsia")]
pub fn current_exe() -> io::Result<PathBuf> {
    use io::ErrorKind;
    Err(io::Error::new(ErrorKind::Other, "Not yet implemented on fuchsia"))
}

pub struct Env {
    iter: vec::IntoIter<(OsString, OsString)>,
    _dont_send_or_sync_me: PhantomData<*mut ()>,
}

impl Iterator for Env {
    type Item = (OsString, OsString);
    fn next(&mut self) -> Option<(OsString, OsString)> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

#[cfg(target_os = "macos")]
pub unsafe fn environ() -> *mut *const *const c_char {
    extern "C" {
        fn _NSGetEnviron() -> *mut *const *const c_char;
    }
    _NSGetEnviron()
}

#[cfg(not(target_os = "macos"))]
pub unsafe fn environ() -> *mut *const *const c_char {
    extern "C" {
        static mut environ: *const *const c_char;
    }
    &mut environ
}

/// Returns a vector of (variable, value) byte-vector pairs for all the
/// environment variables of the current process.
pub fn env() -> Env {
    unsafe {
        ENV_LOCK.lock();
        let mut environ = *environ();
        if environ == ptr::null() {
            ENV_LOCK.unlock();
            panic!("os::env() failure getting env string from OS: {}",
                   io::Error::last_os_error());
        }
        let mut result = Vec::new();
        while *environ != ptr::null() {
            if let Some(key_value) = parse(CStr::from_ptr(*environ)
                                               .to_bytes()) {
                result.push(key_value);
            }
            environ = environ.offset(1);
        }
        let ret = Env {
            iter: result.into_iter(),
            _dont_send_or_sync_me: PhantomData,
        };
        ENV_LOCK.unlock();
        return ret;
    }

    fn parse(input: &[u8]) -> Option<(OsString, OsString)> {
        // Strategy (copied from glibc): Variable name and value are separated
        // by an ASCII equals sign '='. Since a variable name must not be
        // empty, allow variable names starting with an equals sign. Skip all
        // malformed lines.
        if input.is_empty() {
            return None;
        }
        let pos = memchr::memchr(b'=', &input[1..]).map(|p| p + 1);
        pos.map(|p| {
                    (OsStringExt::from_vec(input[..p].to_vec()),
                     OsStringExt::from_vec(input[p + 1..].to_vec()))
                })
    }
}

pub fn getenv(k: &OsStr) -> io::Result<Option<OsString>> {
    // environment variables with a nul byte can't be set, so their value is
    // always None as well
    let k = CString::new(k.as_bytes())?;
    unsafe {
        ENV_LOCK.lock();
        let s = libc::getenv(k.as_ptr()) as *const _;
        let ret = if s.is_null() {
            None
        } else {
            Some(OsStringExt::from_vec(CStr::from_ptr(s).to_bytes().to_vec()))
        };
        ENV_LOCK.unlock();
        return Ok(ret);
    }
}

pub fn setenv(k: &OsStr, v: &OsStr) -> io::Result<()> {
    let k = CString::new(k.as_bytes())?;
    let v = CString::new(v.as_bytes())?;

    unsafe {
        ENV_LOCK.lock();
        let ret = cvt(libc::setenv(k.as_ptr(), v.as_ptr(), 1)).map(|_| ());
        ENV_LOCK.unlock();
        return ret;
    }
}

pub fn unsetenv(n: &OsStr) -> io::Result<()> {
    let nbuf = CString::new(n.as_bytes())?;

    unsafe {
        ENV_LOCK.lock();
        let ret = cvt(libc::unsetenv(nbuf.as_ptr())).map(|_| ());
        ENV_LOCK.unlock();
        return ret;
    }
}

pub fn page_size() -> usize {
    unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize }
}

pub fn temp_dir() -> PathBuf {
    ::env::var_os("TMPDIR").map(PathBuf::from).unwrap_or_else(|| {
        if cfg!(target_os = "android") {
            PathBuf::from("/data/local/tmp")
        } else {
            PathBuf::from("/tmp")
        }
    })
}

pub fn home_dir() -> Option<PathBuf> {
    return ::env::var_os("HOME")
               .or_else(|| unsafe { fallback() })
               .map(PathBuf::from);

    #[cfg(any(target_os = "android",
              target_os = "ios",
              target_os = "nacl",
              target_os = "emscripten"))]
    unsafe fn fallback() -> Option<OsString> {
        None
    }
    #[cfg(not(any(target_os = "android",
                  target_os = "ios",
                  target_os = "nacl",
                  target_os = "emscripten")))]
    unsafe fn fallback() -> Option<OsString> {
        let amt = match libc::sysconf(libc::_SC_GETPW_R_SIZE_MAX) {
            n if n < 0 => 512 as usize,
            n => n as usize,
        };
        let mut buf = Vec::with_capacity(amt);
        let mut passwd: libc::passwd = mem::zeroed();
        let mut result = ptr::null_mut();
        match libc::getpwuid_r(libc::getuid(),
                               &mut passwd,
                               buf.as_mut_ptr(),
                               buf.capacity(),
                               &mut result) {
            0 if !result.is_null() => {
                let ptr = passwd.pw_dir as *const _;
                let bytes = CStr::from_ptr(ptr).to_bytes().to_vec();
                Some(OsStringExt::from_vec(bytes))
            }
            _ => None,
        }
    }
}

pub fn exit(code: i32) -> ! {
    unsafe { libc::exit(code as c_int) }
}
