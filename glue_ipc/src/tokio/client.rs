use std::path::Path;

use serde::Serialize;
use tokio::net::UnixStream;

pub use crate::error::ClientError;
use crate::tokio::protocol::Protocol;

pub struct Client {
    stream: UnixStream,
}

impl Client {
    pub async fn new(socket_path: &str) -> Result<Self, ClientError> {
        if !Path::new(socket_path).exists() {
            return Err(ClientError::SocketNotFound(socket_path.to_string()));
        }
        let stream = UnixStream::connect(socket_path)
            .await
            .map_err(|err| ClientError::SocketConnectError(err))?;
        Ok(Self { stream })
    }

    pub async fn send<T>(&mut self, command: T) -> Result<(), ClientError>
    where
        T: Serialize,
    {
        let message = bincode::serialize(&command)
            .map_err(|err| ClientError::SerializtionError(err.to_string()))?;

        Protocol::new(&mut self.stream)
            .write_message(&message)
            .await
            .map_err(ClientError::Protocol)?;
        Ok(())
    }

    pub async fn read(&mut self) -> Result<Vec<u8>, ClientError> {
        Protocol::new(&mut self.stream)
            .read_message()
            .await
            .map_err(ClientError::Protocol)
    }
}
