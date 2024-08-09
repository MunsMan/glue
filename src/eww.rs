use std::process::Command;

use crate::audio::AudioSettings;

pub enum EwwVariable {
    Workspace(String),
    Audio(AudioSettings),
}

pub fn eww_update(variable: EwwVariable) -> Result<(), ()> {
    let mut command = Command::new("eww");
    command.arg("update");
    match variable {
        EwwVariable::Workspace(id) => command.arg(&format!("workspace={}", id)),
        EwwVariable::Audio(settings) => command.arg(&format!(
            "audio={}",
            serde_json::to_string(&settings).unwrap()
        )),
    };
    match command.spawn() {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}
