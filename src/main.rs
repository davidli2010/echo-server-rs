extern crate env_logger;
#[macro_use]
extern crate log;
extern crate mio;

use mio::*;
use mio::net::TcpListener;
use std::collections::HashMap;
use std::io::{Read, Result, Write};

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
    let buf = &mut [0u8; 1024];
    let mut num = 0;
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

                    clients.insert(client_id, stream);

                    poll.register(&clients[&client_id],
                                  token,
                                  Ready::readable(),
                                  PollOpt::edge())?;
                }
                Token(client_id) => {
                    if event.readiness().is_readable() {
                        let stream = clients.get_mut(&client_id).unwrap();
                        match stream.read(buf) {
                            Ok(n) => {
                                num = n;
                                trace!("Received {} bytes", n);
                                poll.reregister(
                                    stream,
                                    Token(client_id),
                                    Ready::writable(),
                                    PollOpt::edge(),
                                )?;
                            }
                            Err(e) => error!("read error {}", e)
                        }
                    }
                    if event.readiness().is_writable() {
                        let mut stream = clients.remove(&client_id).unwrap();
                        match stream.write(&buf[..num]) {
                            Ok(n) => trace!("write {} bytes", n),
                            Err(e) => error!("write error {}", e)
                        }
                    }
                }
            }
        }
    }
}
