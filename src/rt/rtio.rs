// Copyright 2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use libc;
use option::*;
use result::*;
use libc::c_int;

use ai = rt::io::net::addrinfo;
use rt::io::IoError;
use super::io::process::ProcessConfig;
use super::io::net::ip::{IpAddr, SocketAddr};
use rt::uv::uvio;
use path::Path;
use super::io::support::PathLike;
use super::io::{SeekStyle};
use super::io::{FileMode, FileAccess, FileStat};

// XXX: ~object doesn't work currently so these are some placeholder
// types to use instead
pub type EventLoopObject = uvio::UvEventLoop;
pub type RemoteCallbackObject = uvio::UvRemoteCallback;
pub type IoFactoryObject = uvio::UvIoFactory;
pub type RtioTcpStreamObject = uvio::UvTcpStream;
pub type RtioTcpAcceptorObject = uvio::UvTcpAcceptor;
pub type RtioTcpListenerObject = uvio::UvTcpListener;
pub type RtioUdpSocketObject = uvio::UvUdpSocket;
pub type RtioTimerObject = uvio::UvTimer;
pub type PausibleIdleCallback = uvio::UvPausibleIdleCallback;
pub type RtioPipeObject = uvio::UvPipeStream;
pub type RtioProcessObject = uvio::UvProcess;
pub type RtioUnixListenerObject = uvio::UvUnixListener;
pub type RtioUnixAcceptorObject = uvio::UvUnixAcceptor;
pub type RtioTTYObject = uvio::UvTTY;

pub trait EventLoop {
    fn run(&mut self);
    fn callback(&mut self, ~fn());
    fn pausible_idle_callback(&mut self) -> ~PausibleIdleCallback;
    fn callback_ms(&mut self, ms: u64, ~fn());
    fn remote_callback(&mut self, ~fn()) -> ~RemoteCallbackObject;
    /// The asynchronous I/O services. Not all event loops may provide one
    fn io<'a>(&'a mut self) -> Option<&'a mut IoFactoryObject>;
}

pub trait RemoteCallback {
    /// Trigger the remote callback. Note that the number of times the
    /// callback is run is not guaranteed. All that is guaranteed is
    /// that, after calling 'fire', the callback will be called at
    /// least once, but multiple callbacks may be coalesced and
    /// callbacks may be called more often requested. Destruction also
    /// triggers the callback.
    fn fire(&mut self);
}

/// Data needed to make a successful open(2) call
/// Using unix flag conventions for now, which happens to also be what's supported
/// libuv (it does translation to windows under the hood).
pub struct FileOpenConfig {
    /// Path to file to be opened
    path: Path,
    /// Flags for file access mode (as per open(2))
    flags: int,
    /// File creation mode, ignored unless O_CREAT is passed as part of flags
    priv mode: int
}

pub trait IoFactory {
    fn tcp_connect(&mut self, addr: SocketAddr) -> Result<~RtioTcpStreamObject, IoError>;
    fn tcp_bind(&mut self, addr: SocketAddr) -> Result<~RtioTcpListenerObject, IoError>;
    fn udp_bind(&mut self, addr: SocketAddr) -> Result<~RtioUdpSocketObject, IoError>;
    fn timer_init(&mut self) -> Result<~RtioTimerObject, IoError>;
    fn fs_from_raw_fd(&mut self, fd: c_int, close_on_drop: bool) -> ~RtioFileStream;
    fn fs_open<P: PathLike>(&mut self, path: &P, fm: FileMode, fa: FileAccess)
        -> Result<~RtioFileStream, IoError>;
    fn fs_unlink<P: PathLike>(&mut self, path: &P) -> Result<(), IoError>;
    fn get_host_addresses(&mut self, host: Option<&str>, servname: Option<&str>,
                          hint: Option<ai::Hint>) -> Result<~[ai::Info], IoError>;
    fn fs_stat<P: PathLike>(&mut self, path: &P) -> Result<FileStat, IoError>;
    fn fs_mkdir<P: PathLike>(&mut self, path: &P) -> Result<(), IoError>;
    fn fs_rmdir<P: PathLike>(&mut self, path: &P) -> Result<(), IoError>;
    fn fs_readdir<P: PathLike>(&mut self, path: &P, flags: c_int) ->
        Result<~[Path], IoError>;
    fn spawn(&mut self, config: ProcessConfig)
            -> Result<(~RtioProcessObject, ~[Option<~RtioPipeObject>]), IoError>;

    fn unix_bind<P: PathLike>(&mut self, path: &P) ->
        Result<~RtioUnixListenerObject, IoError>;
    fn unix_connect<P: PathLike>(&mut self, path: &P) ->
        Result<~RtioPipeObject, IoError>;
    fn tty_open(&mut self, fd: c_int, readable: bool, close_on_drop: bool)
            -> Result<~RtioTTYObject, IoError>;
}

pub trait RtioTcpListener : RtioSocket {
    fn listen(self) -> Result<~RtioTcpAcceptorObject, IoError>;
}

pub trait RtioTcpAcceptor : RtioSocket {
    fn accept(&mut self) -> Result<~RtioTcpStreamObject, IoError>;
    fn accept_simultaneously(&mut self) -> Result<(), IoError>;
    fn dont_accept_simultaneously(&mut self) -> Result<(), IoError>;
}

pub trait RtioTcpStream : RtioSocket {
    fn read(&mut self, buf: &mut [u8]) -> Result<uint, IoError>;
    fn write(&mut self, buf: &[u8]) -> Result<(), IoError>;
    fn peer_name(&mut self) -> Result<SocketAddr, IoError>;
    fn control_congestion(&mut self) -> Result<(), IoError>;
    fn nodelay(&mut self) -> Result<(), IoError>;
    fn keepalive(&mut self, delay_in_seconds: uint) -> Result<(), IoError>;
    fn letdie(&mut self) -> Result<(), IoError>;
}

pub trait RtioSocket {
    fn socket_name(&mut self) -> Result<SocketAddr, IoError>;
}

pub trait RtioUdpSocket : RtioSocket {
    fn recvfrom(&mut self, buf: &mut [u8]) -> Result<(uint, SocketAddr), IoError>;
    fn sendto(&mut self, buf: &[u8], dst: SocketAddr) -> Result<(), IoError>;

    fn join_multicast(&mut self, multi: IpAddr) -> Result<(), IoError>;
    fn leave_multicast(&mut self, multi: IpAddr) -> Result<(), IoError>;

    fn loop_multicast_locally(&mut self) -> Result<(), IoError>;
    fn dont_loop_multicast_locally(&mut self) -> Result<(), IoError>;

    fn multicast_time_to_live(&mut self, ttl: int) -> Result<(), IoError>;
    fn time_to_live(&mut self, ttl: int) -> Result<(), IoError>;

    fn hear_broadcasts(&mut self) -> Result<(), IoError>;
    fn ignore_broadcasts(&mut self) -> Result<(), IoError>;
}

pub trait RtioTimer {
    fn sleep(&mut self, msecs: u64);
}

pub trait RtioFileStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<int, IoError>;
    fn write(&mut self, buf: &[u8]) -> Result<(), IoError>;
    fn pread(&mut self, buf: &mut [u8], offset: u64) -> Result<int, IoError>;
    fn pwrite(&mut self, buf: &[u8], offset: u64) -> Result<(), IoError>;
    fn seek(&mut self, pos: i64, whence: SeekStyle) -> Result<u64, IoError>;
    fn tell(&self) -> Result<u64, IoError>;
    fn flush(&mut self) -> Result<(), IoError>;
}

pub trait RtioProcess {
    fn id(&self) -> libc::pid_t;
    fn kill(&mut self, signal: int) -> Result<(), IoError>;
    fn wait(&mut self) -> int;
}

pub trait RtioPipe {
    fn read(&mut self, buf: &mut [u8]) -> Result<uint, IoError>;
    fn write(&mut self, buf: &[u8]) -> Result<(), IoError>;
}

pub trait RtioUnixListener {
    fn listen(self) -> Result<~RtioUnixAcceptorObject, IoError>;
}

pub trait RtioUnixAcceptor {
    fn accept(&mut self) -> Result<~RtioPipeObject, IoError>;
    fn accept_simultaneously(&mut self) -> Result<(), IoError>;
    fn dont_accept_simultaneously(&mut self) -> Result<(), IoError>;
}

pub trait RtioTTY {
    fn read(&mut self, buf: &mut [u8]) -> Result<uint, IoError>;
    fn write(&mut self, buf: &[u8]) -> Result<(), IoError>;
    fn set_raw(&mut self, raw: bool) -> Result<(), IoError>;
    fn reset_mode(&mut self);
    fn get_winsize(&mut self) -> Result<(int, int), IoError>;
}
