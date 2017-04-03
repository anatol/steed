use std::io::{Read, Write};
use std::net::{SocketAddr, IpAddr, Ipv4Addr, TcpStream, TcpListener};

fn main () {
    let localhost = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let addr = SocketAddr::new(localhost, 12345);
    let listener = TcpListener::bind(addr).expect("bind");

    let mut stream = TcpStream::connect(addr).expect("connect");
    let request = b"Hello, World!!!";
    stream.write(request).expect("write");
    drop(stream);

    let (mut accepted, _peer_addr) = listener.accept().expect("accept");
    let mut response = Vec::new();
    accepted.read_to_end(&mut response).expect("read_to_end");
    assert_eq!(request.to_vec(), response);
}
