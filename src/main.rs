extern crate byteorder;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate mio;

use conn::Conn;
use mio::*;
use mio::net::TcpListener;
use msg::*;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::io::Result;

pub mod msg;
pub mod conn;

fn main() -> Result<()> {
    env_logger::init();

    const SERVER: Token = Token(0);
    let mut client_id_counter = 1;

    let addr = "127.0.0.1:8000".parse().unwrap();
    let server = TcpListener::bind(&addr)?;
    let poll = Poll::new()?;

    poll.register(&server,
                  SERVER,
                  Ready::readable(),
                  PollOpt::edge())?;
    let mut events = Events::with_capacity(1024);
    let mut client_msg = String::new();
    let mut clients = HashMap::new();

    info!("Server is running");

    loop {
        poll.poll(&mut events, None)?;

        for event in &events {
            match event.token() {
                SERVER => {
                    let (stream, _) = server.accept()?;
                    let client_id = client_id_counter;
                    client_id_counter += 1;
                    let token = Token(client_id);
                    let connection = Conn::new(stream, token);
                    trace!("Connection {}", connection.token().0);

                    clients.insert(client_id, connection);

                    clients[&client_id].register(&poll, Ready::readable())?;
                }
                Token(client_id) => {
                    let mut disconnect = false;
                    if event.readiness().is_readable() {
                        let connection = clients.get_mut(&client_id).unwrap();
                        let msg = match connection.read() {
                            Ok(msg) => msg,
                            Err(e) => {
                                if e.kind() == ErrorKind::WouldBlock {
                                    trace!("Read block");
                                    connection.reregister(&poll, Ready::readable())?;
                                    continue;
                                } else {
                                    return Err(e);
                                }
                            }
                        };
                        match msg {
                            Msg::Message { header, message } => {
                                trace!("Received {} bytes", header.length());
                                client_msg = message;
                                connection.reregister(&poll, Ready::writable())?;
                            }
                            Msg::Disconnect { header: _ } => {
                                trace!("Received disconnect msg");
                                disconnect = true;
                            }
                        }
                    }
                    if disconnect {
                        let mut connection = clients.remove(&client_id).unwrap();
                        trace!("Disconnect {}", connection.token().0);
                        continue;
                    }
                    if event.readiness().is_writable() {
                        let mut connection = clients.get_mut(&client_id).unwrap();
                        let msg = Msg::new_message(client_msg.to_owned());
                        connection.write(&msg)?;
                        trace!("Write {} bytes", msg.length());
                        connection.reregister(&poll, Ready::readable())?;
                    }
                }
            }
        }
    }
}
