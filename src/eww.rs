use std::process::Command;

use crate::audio::AudioSettings;
use crate::brightness::BrightnessSettings;
use crate::coffee::CoffeeResponse;
use crate::error::CommandError;
use crate::mic::MicSettings;

pub enum EwwVariable {
    Workspace(String),
    Audio(AudioSettings),
    Mic(MicSettings),
    Coffee(CoffeeResponse),
    Brightness(BrightnessSettings),
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
        EwwVariable::Coffee(coffee_response) => format!(
            "coffee={}",
            serde_json::to_string(&coffee_response).unwrap()
        ),
        EwwVariable::Brightness(settings) => {
            format!("bright={}", serde_json::to_string(&settings).unwrap())
        }
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

pub fn open(window_name: &WindowName, config: Option<String>) -> Result<(), CommandError> {
    let mut command = Command::new("eww");
    command.arg("open");
    command.arg(Into::<&str>::into(window_name));
    if let Some(config) = config {
        command.arg(format!("--config={}", config));
    }
    command.spawn().map(|_| ()).map_err(|x| {
        CommandError::Command(
            format!("eww open {}", Into::<&str>::into(window_name)),
            x.to_string(),
        )
    })
}
