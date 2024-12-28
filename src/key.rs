use crate::error::GlueError;

pub(crate) trait FunctionKey {
    fn increase() -> Result<(), GlueError>;
    fn decrease() -> Result<(), GlueError>;
}

pub(crate) trait MuteKey {
    fn mute() -> Result<(), GlueError>;
}
