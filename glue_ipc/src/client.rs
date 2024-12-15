use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::Path;

use serde::Serialize;

pub use crate::error::ClientError;

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
        self.stream
            .write(
                &bincode::serialize(&command)
                    .map_err(|err| ClientError::SerializtionError(err.to_string()))?,
            )
            .map_err(|err| ClientError::SocketWriteError(err))?;
        Ok(())
    }
}
