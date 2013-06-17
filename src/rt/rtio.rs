// Copyright 2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use option::*;
use result::*;

use rt::io::IoError;
use super::io::net::ip::IpAddr;
use rt::uv::uvio;

// XXX: ~object doesn't work currently so these are some placeholder
// types to use instead
pub type EventLoopObject = uvio::UvEventLoop;
pub type RemoteCallbackObject = uvio::UvRemoteCallback;
pub type IoFactoryObject = uvio::UvIoFactory;
pub type RtioTcpStreamObject = uvio::UvTcpStream;
pub type RtioTcpListenerObject = uvio::UvTcpListener;
pub type RtioUdpStreamObject = uvio::UvUdpStream;

pub trait EventLoop {
    fn run(&mut self);
    fn callback(&mut self, ~fn());
    fn callback_ms(&mut self, ms: u64, ~fn());
    fn remote_callback(&mut self, ~fn()) -> ~RemoteCallbackObject;
    /// The asynchronous I/O services. Not all event loops may provide one
    fn io<'a>(&'a mut self) -> Option<&'a mut IoFactoryObject>;
}

pub trait RemoteCallback {
    /// Trigger the remote callback. Note that the number of times the callback
    /// is run is not guaranteed. All that is guaranteed is that, after calling 'fire',
    /// the callback will be called at least once, but multiple callbacks may be coalesced
    /// and callbacks may be called more often requested. Destruction also triggers the
    /// callback.
    fn fire(&mut self);
}

pub trait IoFactory {
    fn tcp_connect(&mut self, addr: IpAddr) -> Result<~RtioTcpStreamObject, IoError>;
    fn tcp_bind(&mut self, addr: IpAddr) -> Result<~RtioTcpListenerObject, IoError>;
    // TODO fn udp_connect(&mut self, addr: IpAddr) -> Result<~RtioUdpStreamObject, IoError>;
}

pub trait RtioTcpListener {
    fn accept(&mut self) -> Result<~RtioTcpStreamObject, IoError>;
}

pub trait RtioTcpStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<uint, IoError>;
    fn write(&mut self, buf: &[u8]) -> Result<(), IoError>;
}

pub trait RtioUdpStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<uint, IoError>;
    fn write(&mut self, buf: &[u8]) -> Result<(), IoError>;
}
