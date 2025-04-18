use log::error;
use serde::de::DeserializeOwned;
use std::fs;
use std::os::unix::net::UnixListener;
use std::path::Path;

pub use crate::error::ServerError;
use crate::protocol::Protocol;

pub struct Server {
    listener: UnixListener,
}

impl Server {
    pub fn new(socket_path: &str) -> Result<Self, ServerError> {
        if Path::new(socket_path).exists() {
            fs::remove_file(socket_path).unwrap();
        }
        let listener = UnixListener::bind(socket_path)
            .map_err(|err| ServerError::SocketCreationError(socket_path.to_string(), err))?;
        Ok(Self { listener })
    }

    pub fn listen<F, T, S>(self, handler: F, mut state: S)
    where
        T: DeserializeOwned,
        S: Clone,
        F: Fn(T, &mut S, Protocol) -> (),
    {
        for stream in self.listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut protocol = Protocol::new(&mut stream);
                    match protocol.read_message() {
                        Ok(buffer) => {
                            let result = bincode::deserialize::<T>(&buffer[..]);
                            match result {
                                Ok(command) => handler(command, &mut state, protocol),
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
