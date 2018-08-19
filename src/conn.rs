use bytes::{BytesMut};
use mio::{Poll, PollOpt, Ready, Token};
use mio::net::TcpStream;
use msg::{self, Codec, Msg, MsgHeader};
use std::io::{Read, Result};

const DEFAULT_BUFFER_SIZE: usize = 1024 * 4;

enum ReadState {
    Init,
    Header,
    Body(usize),
}

pub struct Conn {
    stream: TcpStream,
    token: Token,
    buffer: BytesMut,
    buf_len: usize,
    read_state: ReadState,
    message: Option<String>,
}

impl Conn {
    pub fn new(stream: TcpStream, token: Token) -> Conn {
        Conn {
            stream,
            token,
            buffer: BytesMut::with_capacity(DEFAULT_BUFFER_SIZE),
            buf_len: 0,
            read_state: ReadState::Init,
            message: None,
        }
    }

    pub fn token(&self) -> Token {
        self.token
    }

    pub fn save_message(&mut self, message: String) {
        self.message = Some(message)
    }

    pub fn take_message(&mut self) -> Option<String> {
        self.message.take()
    }

    pub fn register(&self, poll: &Poll, interest: Ready) -> Result<()> {
        poll.register(&self.stream,
                      self.token,
                      interest,
                      PollOpt::edge() | PollOpt::oneshot(),
        )
    }

    pub fn reregister(&self, poll: &Poll, interest: Ready) -> Result<()> {
        poll.reregister(&self.stream,
                        self.token,
                        interest,
                        PollOpt::edge() | PollOpt::oneshot(),
        )
    }

    pub fn write(&mut self, message: &Msg) -> Result<()> {
        message.write(&mut self.stream)
    }

    pub fn read(&mut self) -> Result<Msg> {
        loop {
            match self.read_state {
                ReadState::Init => {
                    let buf_cap = self.buffer.capacity();
                    unsafe { self.buffer.set_len(buf_cap) };
                    self.buf_len = 0;
                    self.read_state = ReadState::Header;
                }
                ReadState::Header => {
                    let buf_len = self.buf_len;
                    if buf_len < msg::msg_header_length() {
                        self.read_exact(msg::msg_header_length() - buf_len)?;
                    } else {
                        let mut buf = &self.buffer[0..msg::msg_header_length()];
                        let header = MsgHeader::read(&mut buf)?;
                        self.read_state = ReadState::Body(header.length() as usize);
                    }
                }
                ReadState::Body(length) => {
                    let buf_len = self.buf_len;
                    if buf_len < length {
                        self.read_exact(length - buf_len)?;
                    } else {
                        let mut buf = &self.buffer[0..length];
                        let msg = Msg::read(&mut buf)?;
                        self.read_state = ReadState::Init;
                        return Ok(msg);
                    }
                }
            }
        }
    }

    fn read_exact(&mut self, len: usize) -> Result<()> {
        let buf_len = self.buf_len;
        let buf_cap = self.buffer.capacity();
        if buf_len + len > buf_cap {
            self.buffer.reserve(buf_len + len - buf_cap);
            unsafe { self.buffer.set_len(buf_len + len) }
        }

        let mut buf = &mut self.buffer[buf_len..buf_len + len];
        let size = self.stream.read(&mut buf)?;
        self.buf_len += size;
        Ok(())
    }
}