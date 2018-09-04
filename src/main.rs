extern crate byteorder;
extern crate bytes;
extern crate env_logger;
extern crate futures;
#[macro_use]
extern crate log;
extern crate tokio;
extern crate tokio_codec;
extern crate tokio_io;

use server::Server;
use std::io::Result;

pub mod msg;
pub mod server;

fn main() -> Result<()> {
    env_logger::init();

    let server = Server::new("127.0.0.1:8000")?;

    server.run()
}
