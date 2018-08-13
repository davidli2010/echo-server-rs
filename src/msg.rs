use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Result, Write};
use std::io::{Error, ErrorKind};
use std::mem::size_of;

pub trait Codec {
    fn read(buffer: &mut impl Read) -> Result<Self> where Self: Sized;
    fn write(&self, buffer: &mut impl Write) -> Result<()>;
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub enum MsgCode {
    Message = 1,
    Disconnect = 2,
}

impl MsgCode {
    fn from(u: u32) -> Option<MsgCode> {
        match u {
            1 => Some(MsgCode::Message),
            2 => Some(MsgCode::Disconnect),
            _ => None,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct MsgHeader {
    length: u32,
    code: MsgCode,
}

impl MsgHeader {
    fn new(code: MsgCode, length: u32) -> MsgHeader {
        MsgHeader {
            length,
            code,
        }
    }

    #[inline]
    pub fn code(&self) -> MsgCode {
        self.code
    }

    #[inline]
    pub fn length(&self) -> u32 {
        self.length
    }
}

impl Codec for MsgHeader {
    fn read(buffer: &mut impl Read) -> Result<MsgHeader> {
        let length = buffer.read_u32::<LittleEndian>()?;
        let code_u32 = buffer.read_u32::<LittleEndian>()?;
        let code = match MsgCode::from(code_u32) {
            None =>
                return Err(Error::from(ErrorKind::InvalidData)),
            Some(code) => code
        };

        Ok(MsgHeader::new(code, length))
    }

    fn write(&self, buffer: &mut impl Write) -> Result<()> {
        buffer.write_u32::<LittleEndian>(self.length)?;
        buffer.write_u32::<LittleEndian>(self.code as u32)?;
        Ok(())
    }
}

#[inline]
pub fn msg_header_length() -> usize {
    size_of::<MsgHeader>()
}

#[derive(Debug, Eq, PartialEq)]
pub enum Msg {
    Message {
        header: MsgHeader,
        message: String,
    },
    Disconnect {
        header: MsgHeader,
    },
}

impl Msg {
    pub fn code(&self) -> MsgCode {
        match self {
            Msg::Message { header, message: _ } => header.code(),
            Msg::Disconnect { header } => header.code(),
        }
    }

    pub fn length(&self) -> u32 {
        match self {
            Msg::Message { header, message: _ } => header.length(),
            Msg::Disconnect { header } => header.length(),
        }
    }

    pub fn new_message(message: String) -> Msg {
        let length = msg_header_length() as u32 + message.len() as u32;
        Msg::Message {
            header: MsgHeader::new(MsgCode::Message, length),
            message,
        }
    }

    pub fn new_disconnect() -> Msg {
        let length = msg_header_length() as u32;
        Msg::Disconnect {
            header: MsgHeader::new(MsgCode::Disconnect, length),
        }
    }

    fn write_message(buffer: &mut impl Write, header: &MsgHeader, message: &str) -> Result<()> {
        header.write(buffer)?;
        buffer.write_all(message.as_bytes())?;
        Ok(())
    }

    fn write_disconnect(buffer: &mut impl Write, header: &MsgHeader) -> Result<()> {
        header.write(buffer)
    }

    fn read_message(header: MsgHeader, buffer: &mut impl Read) -> Result<Msg> {
        let str_len = header.length as usize - msg_header_length();
        //let mut message = String::with_capacity(str_len);
        let mut buf = Vec::with_capacity(str_len);
        unsafe { buf.set_len(str_len) }
        buffer.read_exact(&mut buf)?;
        if buf.len() != str_len {
            return Err(Error::from(ErrorKind::InvalidData));
        }
        Ok(Msg::Message {
            header,
            message: unsafe { String::from_utf8_unchecked(buf) },
        })
    }
}

impl Codec for Msg {
    fn read(buffer: &mut impl Read) -> Result<Msg> {
        let header = MsgHeader::read(buffer)?;
        match header.code() {
            MsgCode::Message => Msg::read_message(header, buffer),
            MsgCode::Disconnect => Ok(Msg::Disconnect { header }),
        }
    }

    fn write(&self, buffer: &mut impl Write) -> Result<()> {
        match self {
            Msg::Message { header, message } => Msg::write_message(buffer, header, message)?,
            Msg::Disconnect { header } => Msg::write_disconnect(buffer, header)?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_msg_header() {
        let length = size_of::<MsgHeader>() as u32;
        let msg = MsgHeader::new(MsgCode::Disconnect, length);

        assert_eq!(msg.length(), length);
        assert_eq!(msg.code(), MsgCode::Disconnect);
    }

    #[test]
    fn test_message() {
        let msg_str = "hello".to_string();
        let length = msg_str.len() as u32 + size_of::<MsgHeader>() as u32;
        let msg = Msg::new_message(msg_str);
        assert_eq!(msg.length(), length);
        assert_eq!(msg.code(), MsgCode::Message);
    }
}
