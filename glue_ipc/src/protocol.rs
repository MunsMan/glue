use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
};

use crate::error::ProtocolError;

pub struct Protocol<'a>(&'a mut UnixStream);

impl<'a> Protocol<'a> {
    pub fn new(stream: &'a mut UnixStream) -> Self {
        Self(stream)
    }

    pub fn write_message(&mut self, message: &[u8]) -> Result<(), ProtocolError> {
        let payload_len = message.len() as u32;
        self.0
            .write_u32::<BigEndian>(payload_len)
            .map_err(ProtocolError::SocketWriteError)?;
        self.0
            .write_all(message)
            .map_err(ProtocolError::SocketWriteError)?;
        self.0.flush().map_err(ProtocolError::SocketWriteError)?;
        Ok(())
    }

    pub fn read_message(&mut self) -> Result<Vec<u8>, ProtocolError> {
        let payload_len = self
            .0
            .read_u32::<BigEndian>()
            .map_err(ProtocolError::SocketReadError)?;
        let mut payload = vec![0; payload_len as usize];
        self.0
            .read_exact(&mut payload)
            .map_err(ProtocolError::SocketReadError)?;
        Ok(payload)
    }
}
