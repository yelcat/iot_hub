use std::io;
use std::net::SocketAddr;

use futures::{Future, Stream, Sink};
use tokio_core::net::{UdpSocket, UdpCodec};
use tokio_core::reactor::Core;

use proto::{Message, MessageCodec};

pub struct Server {
}

impl Server {
    pub fn new(address: &str) {
        let mut core = Core::new().unwrap();
        let handle = core.handle();

        let addr: SocketAddr = address.parse().unwrap();
        let socket = UdpSocket::bind(&addr, &handle).unwrap();
        println!("Listening on: {}", socket.local_addr().unwrap());

        let (sink, stream) = socket.framed(MessageCodec).split();
        let server = stream.for_each(|(_addr, msg)| {
            if let Message::Ack(text) = msg {
                println!("recv: {}", text);
            }
            Ok(())
        }); 
        core.run(server).unwrap();
    }
}

