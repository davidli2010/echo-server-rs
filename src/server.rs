use conn::Conn;
use mio::{Events, Poll, PollOpt, Ready, Token};
use mio::net::TcpListener;
use msg::Msg;
use std::collections::HashMap;
use std::io::{ErrorKind, Result};

const SERVER: Token = Token(0);

enum ReadState {
    Message(Msg),
    WouldBlock,
}

pub struct Server {
    listener: TcpListener,
    poll: Poll,
}

impl Server {
    pub fn new(url: &str) -> Result<Server> {
        let addr = url.parse().unwrap();
        let listener = TcpListener::bind(&addr)?;
        let poll = Poll::new()?;

        Ok(Server {
            listener,
            poll,
        })
    }

    pub fn run(&self) -> Result<()> {
        let mut client_id_counter = 1;
        let mut events = Events::with_capacity(1024);
        let mut client_msg = String::new();
        let mut clients = HashMap::new();

        self.poll.register(&self.listener,
                           SERVER,
                           Ready::readable(),
                           PollOpt::edge())?;

        info!("Server is running");

        loop {
            self.poll.poll(&mut events, None)?;

            for event in &events {
                match event.token() {
                    SERVER => {
                        self.accept(&mut client_id_counter, &mut clients)?;
                    }
                    Token(client_id) => {
                        let mut disconnect = false;
                        if event.readiness().is_readable() {
                            let connection = clients.get_mut(&client_id).unwrap();
                            let msg = match self.read(connection)? {
                                ReadState::Message(msg) => msg,
                                ReadState::WouldBlock => continue,
                            };
                            match msg {
                                Msg::Message { header, message } => {
                                    trace!("[{}] Received {} bytes", client_id, header.length());
                                    client_msg = message;
                                    connection.reregister(&self.poll, Ready::writable())?;
                                }
                                Msg::Disconnect { header: _ } => {
                                    trace!("[{}] Received disconnect msg", client_id);
                                    disconnect = true;
                                }
                            }
                        }
                        if disconnect {
                            let mut _connection = clients.remove(&client_id).unwrap();
                            trace!("[{}] Disconnect", client_id);
                            continue;
                        }
                        if event.readiness().is_writable() {
                            let mut connection = clients.get_mut(&client_id).unwrap();
                            self.write(connection, &client_msg)?;
                        }
                    }
                }
            }
        }
    }

    fn accept(&self, client_id_counter: &mut usize, clients: &mut HashMap<usize, Conn>) -> Result<()> {
        let (stream, _) = self.listener.accept()?;
        let client_id = *client_id_counter;
        *client_id_counter += 1;
        let token = Token(client_id);
        let connection = Conn::new(stream, token);
        trace!("[{}] Connection accepted", token.0);

        clients.insert(client_id, connection);

        clients[&client_id].register(&self.poll, Ready::readable())
    }

    fn read(&self, connection: &mut Conn) -> Result<ReadState> {
        let client_id = connection.token().0;
        match connection.read() {
            Ok(msg) => Ok(ReadState::Message(msg)),
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    trace!("[{}] Read block", client_id);
                    connection.reregister(&self.poll, Ready::readable())?;
                    return Ok(ReadState::WouldBlock);
                } else {
                    return Err(e);
                }
            }
        }
    }

    fn write(&self, connection: &mut Conn, client_msg: &str) -> Result<()> {
        let msg = Msg::new_message(client_msg.to_string());
        connection.write(&msg)?;
        trace!("[{}] Write {} bytes", connection.token().0, msg.length());
        connection.reregister(&self.poll, Ready::readable())
    }
}
