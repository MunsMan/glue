use std::os::unix::net::UnixStream;
use std::path::Path;

use serde::Serialize;

pub use crate::error::ClientError;
use crate::protocol::Protocol;

pub struct Client {
    stream: UnixStream,
}

impl Client {
    pub fn new(socket_path: &str) -> Result<Self, ClientError> {
        if !Path::new(socket_path).exists() {
            return Err(ClientError::SocketNotFound(socket_path.to_string()));
        }
        let stream =
            UnixStream::connect(socket_path).map_err(|err| ClientError::SocketConnectError(err))?;
        Ok(Self { stream })
    }

    pub fn send<T>(&mut self, command: T) -> Result<(), ClientError>
    where
        T: Serialize,
    {
        let message = bincode::serialize(&command)
            .map_err(|err| ClientError::SerializtionError(err.to_string()))?;

        Protocol::new(&mut self.stream)
            .write_message(&message)
            .map_err(ClientError::Protocol)?;
        Ok(())
    }

    pub fn read(&mut self) -> Result<Vec<u8>, ClientError> {
        Protocol::new(&mut self.stream)
            .read_message()
            .map_err(ClientError::Protocol)
    }
}
