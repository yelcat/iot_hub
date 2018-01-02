use std::io;
use std::net::SocketAddr;
use tokio_core::net::{UdpSocket, UdpCodec, UdpFramed};

#[derive(Debug)]
pub enum Message {
    Connect(String),
    Pub(String),
    Sub(String),
    Unsub(String),
    Ping(String),
    Ack(String),
}

const CONNECT: u8 = 0x01;
const PUB: u8 = 0x02;
const SUB: u8 = 0x03;
const UNSUB: u8 = 0x04;
const MSG: u8 = 0x05;
const PING: u8 = 0x07;
const PONG: u8 = 0x08;
const OK: u8 = 0x09;
const ERR: u8 = 0x10;

const ILLEGAL_CHANNEL_NAME: u8 = 0x01;


#[derive(Default)]
pub struct MessageCodec;

impl UdpCodec for MessageCodec {
    type In = (SocketAddr, Message);
    type Out = (SocketAddr, Message);

    fn decode(&mut self, addr: &SocketAddr, buf: &[u8]) -> io::Result<Self::In> {
        match buf[0] {
            CONNECT => Ok((*addr, Message::Connect(String::from("connect")))),
            PUB => Ok((*addr, Message::Pub(String::from("pub")))),
            _ => panic!(format!("Invalid message type {}", buf[0])),
        }
    } 

    fn encode(&mut self, (addr, msg): Self::Out, into: &mut Vec<u8>) -> SocketAddr {
        //    into.extend([]);
        addr
    }
}
