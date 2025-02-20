use std::error::Error;

pub trait FunctionKey<E: Error> {
    fn increase() -> Result<(), E>;
    fn decrease() -> Result<(), E>;
}

pub trait MuteKey<E: Error> {
    fn mute() -> Result<(), E>;
}

pub trait Changeable<T, E: Error> {
    fn change(&mut self, change: Change<T>) -> Result<(), E>;
}

pub enum Change<T> {
    Add(T),
    Sub(T),
    Absolute(T),
}
