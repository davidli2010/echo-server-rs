use futures::{Future, future, Sink, Stream};
use msg::{Msg, MsgCodec};
use std::io::{Error, ErrorKind, Result};
use tokio;
use tokio::net::{TcpListener, TcpStream};
use tokio_codec::Decoder;

pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub fn new(url: &str) -> Result<Server> {
        let addr = url.parse().unwrap();
        let listener = TcpListener::bind(&addr)?;

        Ok(Server {
            listener,
        })
    }

    pub fn run(self) -> Result<()> {
        let mut client_id_counter = 1usize;
        let server = self.listener.incoming()
            .map_err(|err| {
                error!("Accept error = {:?}", err);
            })
            .for_each(move |socket| {
                Server::process(socket, client_id_counter);
                client_id_counter += 1;
                Ok(())
            });

        info!("Server is running");

        tokio::run(server);

        Ok(())
    }

    fn process(socket: TcpStream, client_id: usize) {
        let frame = MsgCodec::new().framed(socket);
        let (tx, rx) = frame.split();

        let client = tx.send_all(
            rx.and_then(move |msg| {
                if let Msg::Disconnect { header: _ } = msg {
                    trace!("[{}]: Received disconnect msg", client_id);
                    future::err(Error::from(ErrorKind::ConnectionAborted))
                } else {
                    trace!("[{}]: Received {} bytes, msg: {:?}", client_id, msg.length(), msg);
                    future::ok(msg)
                }
            })
        ).then(move |res| {
            if let Err(e) = res {
                if e.kind() == ErrorKind::ConnectionAborted {
                    info!("[{}] Connection disconnect", client_id);
                } else {
                    error!("[{}] Failed to process connection; error = {:?}", client_id, e);
                }
                Err(())
            } else {
                Ok(())
            }
        });

        tokio::spawn(client);
    }
}
