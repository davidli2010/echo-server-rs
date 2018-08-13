extern crate byteorder;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate mio;

use server::Server;
use std::io::Result;

pub mod msg;
pub mod conn;
pub mod server;

fn main() -> Result<()> {
    env_logger::init();

    let mut server = Server::new("127.0.0.1:8000")?;

    server.run()
}
