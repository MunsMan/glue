use crate::{error::GlueError, Change};

pub(crate) trait FunctionKey {
    fn increase() -> Result<(), GlueError>;
    fn decrease() -> Result<(), GlueError>;
}

pub(crate) trait MuteKey {
    fn mute() -> Result<(), GlueError>;
}

pub(crate) trait Changeable<T> {
    fn change(&mut self, change: Change<T>) -> Result<(), GlueError>;
}
