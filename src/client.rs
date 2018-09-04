extern crate byteorder;
extern crate bytes;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate tokio_codec;

use msg::*;
use std::io::Result;
use std::net::TcpStream;

pub mod msg;

fn main() -> Result<()> {
    env_logger::init();

    info!("client is connecting to server");

    let mut stream = TcpStream::connect("127.0.0.1:8000")?;

    info!("client is connected to server");

    for i in 0..10 {
        let msg = Msg::new_message("hello mio".to_string());

        msg.write(&mut stream)?;

        info!("[{}] client write {} bytes: {:?}", i, msg.length(), msg);

        let recv_msg = Msg::read(&mut stream)?;

        info!("[{}] Receive {} bytes: {:?}", i, recv_msg.length(), recv_msg);

        assert_eq!(msg, recv_msg);
    }

    let msg = Msg::new_disconnect();

    msg.write(&mut stream)?;

    info!("Send disconnect msg");

    Ok(())
}