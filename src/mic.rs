use std::process::Command;

use serde::Serialize;

use crate::error::{AudioError, CommandError, ParseError};
use crate::eww::{eww_update, EwwVariable};

#[derive(Serialize, Clone)]
pub struct MicSettings {
    volume: f32,
    mute: bool,
    icon: char,
}

impl MicSettings {
    fn try_new() -> Result<Self, AudioError> {
        let (volume, mute) = get_mic_info()?;
        let icon = Self::icon(mute);
        Ok(Self { volume, mute, icon })
    }

    fn icon(mute: Mute) -> char {
        match mute {
            true => '',
            false => '',
        }
    }

    fn update(&self) -> Result<(), CommandError> {
        eww_update(EwwVariable::Mic(self.clone()))
    }
}

type MicVolumeLevel = f32;
type Mute = bool;

fn get_mic_info() -> Result<(MicVolumeLevel, Mute), AudioError> {
    let output = Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_SOURCE@"])
        .output()
        .map_or_else(
            |_| String::default(),
            |x| String::from_utf8(x.stdout).unwrap(),
        );
    let volume = output.split_whitespace().collect::<Vec<_>>()[1];
    let mute = output.contains("MUTED");
    let volume = volume
        .parse::<f32>()
        .map_err(|x| ParseError::Volume(volume.to_string(), x.to_string()))
        .map_err(AudioError::VolumeParse)?;
    Ok((volume * 100.0, mute))
}

pub fn get_mic() -> Result<(), AudioError> {
    let settings = MicSettings::try_new()?;
    settings.update().map_err(AudioError::Command)
}

pub fn toggle_mic() -> Result<(), AudioError> {
    let _ = Command::new("wpctl")
        .args(["set-mute", "@DEFAULT_SOURCE@", "toggle"])
        .spawn()
        .map_err(|x| CommandError::Command("wpctl set-mute...".to_string(), x.to_string()))
        .map_err(AudioError::Update)
        .map(|_| ());
    let settings = MicSettings::try_new()?;
    settings.update().map_err(AudioError::Command)
}
