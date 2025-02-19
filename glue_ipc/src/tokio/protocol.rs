use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;

use crate::error::ProtocolError;

pub struct Protocol<'a>(&'a mut UnixStream);

impl<'a> Protocol<'a> {
    pub fn new(stream: &'a mut UnixStream) -> Self {
        Self(stream)
    }

    pub async fn write_message(&mut self, message: &[u8]) -> Result<(), ProtocolError> {
        let payload_len = message.len() as u32;
        self.0
            .write_u32(payload_len)
            .await
            .map_err(ProtocolError::SocketWriteError)?;
        self.0
            .write_all(message)
            .await
            .map_err(ProtocolError::SocketWriteError)?;
        self.0
            .flush()
            .await
            .map_err(ProtocolError::SocketWriteError)?;
        Ok(())
    }

    pub async fn read_message(&mut self) -> Result<Vec<u8>, ProtocolError> {
        let payload_len = self
            .0
            .read_u32()
            .await
            .map_err(ProtocolError::SocketReadError)?;
        let mut payload = vec![0; payload_len as usize];
        self.0
            .read_exact(&mut payload)
            .await
            .map_err(ProtocolError::SocketReadError)?;
        Ok(payload)
    }
}
