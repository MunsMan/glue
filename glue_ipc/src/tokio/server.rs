use log::error;
use serde::de::DeserializeOwned;
use std::future::Future;
use std::path::Path;
use tokio::fs;
use tokio::net::{UnixListener, UnixStream};

pub use crate::error::ServerError;
use crate::tokio::protocol::Protocol;

pub struct Server {
    listener: UnixListener,
}

impl Server {
    pub async fn new(socket_path: &str) -> Result<Self, ServerError> {
        if Path::new(socket_path).exists() {
            fs::remove_file(socket_path).await.unwrap();
        }
        let listener = UnixListener::bind(socket_path)
            .map_err(|err| ServerError::SocketCreationError(socket_path.to_string(), err))?;
        Ok(Self { listener })
    }

    pub async fn listen<F, T, S, Fut>(self, handler: F, state: S)
    where
        T: DeserializeOwned,
        S: Clone,
        F: Fn(T, S, UnixStream) -> Fut,
        Fut: Future<Output = ()>,
    {
        loop {
            match self.listener.accept().await {
                Ok((mut stream, _)) => {
                    let mut protocol = Protocol::new(&mut stream);
                    match protocol.read_message().await {
                        Ok(buffer) => {
                            let result = bincode::deserialize::<T>(&buffer[..]);
                            match result {
                                Ok(command) => {
                                    handler(command, state.clone(), stream).await;
                                }
                                Err(err) => error!("Unable to serialize message: {err}"),
                            }
                        }
                        Err(err) => error!("Unable to read Socket: {}", err),
                    }
                }
                Err(_) => todo!(),
            }
        }
    }
}
