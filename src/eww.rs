use std::process::Command;

use crate::audio::AudioSettings;
use crate::battery::Battery;
use crate::error::CommandError;
use crate::mic::MicSettings;

pub enum EwwVariable {
    Workspace(String),
    Audio(AudioSettings),
    Mic(MicSettings),
    Battery(Battery),
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
        EwwVariable::Battery(battery) => {
            format!("battery={}", serde_json::to_string(&battery).unwrap())
        }
    };
    command.arg(&argument);
    command
        .spawn()
        .map(|_| ())
        .map_err(|x| CommandError::Command(format!("eww update {}", argument), x.to_string()))
}
