use log::error;
use serde::de::DeserializeOwned;
use std::io::Read;
use std::os::unix::net::UnixListener;

pub use crate::error::ServerError;

pub struct Server {
    listener: UnixListener,
}

impl Server {
    pub fn new(socket_path: &str) -> Result<Self, ServerError> {
        let listener = UnixListener::bind(socket_path)
            .map_err(|err| ServerError::SocketCreationError(socket_path.to_string(), err))?;
        Ok(Self { listener })
    }

    pub fn listen<F, T, S>(self, handler: F, mut state: S)
    where
        T: DeserializeOwned,
        S: Clone,
        F: Fn(T, &mut S) -> (),
    {
        for stream in self.listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut buffer = Vec::new();
                    match stream.read_to_end(&mut buffer) {
                        Ok(_) => {
                            let result = bincode::deserialize::<T>(&buffer[..]);
                            match result {
                                Ok(command) => handler(command, &mut state),
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
