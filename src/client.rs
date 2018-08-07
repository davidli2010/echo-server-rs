extern crate env_logger;
#[macro_use]
extern crate log;

use std::io::{Read, Write};
use std::net::TcpStream;

fn main() {
    env_logger::init();

    let msg = "client message";
    let mut buf = [0u8; 1024];

    info!("client is connecting to server");

    let mut stream = TcpStream::connect("127.0.0.1:8000").unwrap();

    info!("client is connected to server");

    stream.write_all(msg.as_ref()).unwrap();

    info!("client write {} bytes", msg.len());

    let n = stream.read(&mut buf).unwrap();

    info!("Receive {} bytes: {:?}", n, String::from_utf8_lossy(&buf[..n]));
}