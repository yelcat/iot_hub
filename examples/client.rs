extern crate tokio_core;
extern crate futures;

use std::net::SocketAddr;

use tokio_core::net::UdpSocket;
use tokio_core::reactor::Core;

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let addr: SocketAddr = "127.0.0.1:5515".parse().unwrap();
    let socket = UdpSocket::bind(&addr, &handle).unwrap();
    println!("Listening on: {}", socket.local_addr().unwrap());

    let remote: SocketAddr = "127.0.0.1:5514".parse().unwrap();
    let send = socket.send_dgram(&[0x01], remote);
    core.run(send).unwrap();
}
