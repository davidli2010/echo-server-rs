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
    client_id_counter: usize,
    clients: HashMap<usize, Conn>,
}

impl Server {
    pub fn new(url: &str) -> Result<Server> {
        let addr = url.parse().unwrap();
        let listener = TcpListener::bind(&addr)?;
        let poll = Poll::new()?;

        Ok(Server {
            listener,
            poll,
            client_id_counter: 1,
            clients: HashMap::new(),
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let mut events = Events::with_capacity(1024);

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
                        self.accept()?;
                    }
                    Token(client_id) => {
                        if event.readiness().is_readable() {
                            let msg = match self.read(client_id)? {
                                ReadState::Message(msg) => msg,
                                ReadState::WouldBlock => continue,
                            };
                            self.process_msg(msg, client_id)?;
                        }
                        if event.readiness().is_writable() {
                            self.send_message(client_id)?;
                            self.reregister_client(client_id, Ready::readable())?;
                        }
                    }
                }
            }
        }
    }

    fn accept(&mut self) -> Result<()> {
        let (stream, _) = self.listener.accept()?;
        let client_id = self.client_id_counter;
        self.client_id_counter += 1;
        let token = Token(client_id);
        let client = Conn::new(stream, token);

        trace!("[{}] Client accepted", token.0);

        client.register(&self.poll, Ready::readable())?;
        self.clients.insert(client_id, client);

        Ok(())
    }

    fn read(&mut self, client_id: usize) -> Result<ReadState> {
        let client = self.clients.get_mut(&client_id).unwrap();
        match client.read() {
            Ok(msg) => Ok(ReadState::Message(msg)),
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    trace!("[{}] Read block", client_id);
                    client.reregister(&self.poll, Ready::readable())?;
                    return Ok(ReadState::WouldBlock);
                } else {
                    return Err(e);
                }
            }
        }
    }

    fn process_msg(&mut self, msg: Msg, client_id: usize) -> Result<()> {
        match msg {
            Msg::Message { header, message } => {
                trace!("[{}] Received {} bytes", client_id, header.length());
                self.save_message(client_id, message);
                self.reregister_client(client_id, Ready::writable())?;
            }
            Msg::Disconnect { header: _ } => {
                trace!("[{}] Received disconnect msg", client_id);
                self.remove_client(client_id);
            }
        }

        Ok(())
    }

    fn reregister_client(&self, client_id: usize, interest: Ready) -> Result<()> {
        self.clients[&client_id].reregister(&self.poll, interest)
    }

    fn remove_client(&mut self, client_id: usize) {
        let mut _client = self.clients.remove(&client_id);
        trace!("[{}] Disconnect", client_id);
    }

    fn save_message(&mut self, client_id: usize, client_msg: String) {
        let client = self.clients.get_mut(&client_id).unwrap();
        client.save_message(client_msg)
    }

    fn send_message(&mut self, client_id: usize) -> Result<()> {
        let client = self.clients.get_mut(&client_id).unwrap();
        if let Some(message) = client.take_message() {
            let msg = Msg::new_message(message);
            client.write(&msg)?;
            trace!("[{}] Write {} bytes", client.token().0, msg.length());
        }

        Ok(())
    }
}
