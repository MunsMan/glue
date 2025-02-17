use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Unable to connect to socket: {}", .0)]
    SocketConnectError(std::io::Error),
    #[error("Unable to Serialize: {}", .0)]
    SerializtionError(String),
    #[error("Unable to Serialize: {}", .0)]
    SocketWriteError(std::io::Error),
    #[error("Unable to find the socket: {}", .0)]
    SocketNotFound(String),
    #[error("Socket Reading Error: {}", .0)]
    SocketReadError(String),
    #[error("Protocol Error: {}", .0)]
    Protocol(ProtocolError),
}

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Unable to setup the socket {}: {}", .0, .1)]
    SocketCreationError(String, std::io::Error),
}

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Unable to write to the socket: {}", .0)]
    SocketWriteError(std::io::Error),
    #[error("Unable to read to the socket: {}", .0)]
    SocketReadError(std::io::Error),
}
