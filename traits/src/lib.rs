use serde::de::DeserializeOwned;
use std::{error::Error, os::unix::net::UnixStream};
use tokio::net::UnixStream as AsyncUnixStream;

pub trait FunctionKey<E: Error> {
    fn increase() -> Result<(), E>;
    fn decrease() -> Result<(), E>;
}

pub trait ToggleKey<E: Error> {
    fn toggle(&mut self) -> Result<(), E>;
}

pub trait Changeable<T, E: Error> {
    fn change(&mut self, change: Change<T>) -> Result<(), E>;
}

pub enum Change<T> {
    Add(T),
    Sub(T),
    Absolute(T),
}

pub trait Listener<C, S, E>
where
    C: DeserializeOwned,
    S: Clone,
{
    fn listener(command: C, state: S, stream: UnixStream) -> Result<(), E>;
}

pub trait AsyncListener<C, S, E>
where
    C: DeserializeOwned,
    S: Clone,
{
    fn listener(
        command: C,
        state: S,
        stream: AsyncUnixStream,
    ) -> impl std::future::Future<Output = Result<(), E>> + Send;
}
