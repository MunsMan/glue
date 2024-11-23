use std::process::Command;

use crate::audio::AudioSettings;
use crate::error::CommandError;
use crate::mic::MicSettings;

pub enum EwwVariable {
    Workspace(String),
    Audio(AudioSettings),
    Mic(MicSettings),
}

pub fn eww_update(variable: EwwVariable) -> Result<(), CommandError> {
    let mut command = Command::new("eww");
    command.arg("update");
    let argument = match variable {
        EwwVariable::Workspace(id) => format!("workspace={}", id),
        EwwVariable::Audio(settings) => {
            format!("audio={}", serde_json::to_string(&settings).unwrap())
        }
        EwwVariable::Mic(settings) => format!("mic={}", serde_json::to_string(&settings).unwrap()),
    };
    command.arg(&argument);
    command
        .spawn()
        .map(|_| ())
        .map_err(|x| CommandError::Command(format!("eww update {}", argument), x.to_string()))
}

pub enum WindowName {
    Bar,
}

impl From<&WindowName> for &str {
    fn from(val: &WindowName) -> Self {
        match val {
            WindowName::Bar => "bar",
        }
    }
}

pub fn open(window_name: &WindowName) -> Result<(), CommandError> {
    let mut command = Command::new("eww");
    command.arg("open");
    command.arg(Into::<&str>::into(window_name));
    command.spawn().map(|_| ()).map_err(|x| {
        CommandError::Command(
            format!("eww open {}", Into::<&str>::into(window_name)),
            x.to_string(),
        )
    })
}
