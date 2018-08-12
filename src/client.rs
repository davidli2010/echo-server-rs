extern crate byteorder;
extern crate env_logger;
#[macro_use]
extern crate log;

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

        info!("[{}]: client write {} bytes", i, msg.length());

        let m = Msg::read(&mut stream)?;

        info!("[{}]: Receive {} bytes", i, m.length());
    }

    let msg = Msg::new_disconnect();

    msg.write(&mut stream)?;

    info!("Send disconnect msg");

    Ok(())
}