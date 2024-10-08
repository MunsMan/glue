use std::process::Command;

use serde::Serialize;

use crate::error::{AudioError, CommandError, ParseError};
use crate::eww::{eww_update, EwwVariable};

#[derive(Serialize, Clone, Debug)]
pub struct AudioSettings {
    volume: f32,
    mute: bool,
    icon: char,
}

impl AudioSettings {
    fn try_new() -> Result<Self, AudioError> {
        let (volume, mute) = get_volume()?;
        Ok(Self::from_volume(volume, mute))
    }

    fn from_volume(volume: VolumeLevel, mute: Mute) -> Self {
        let headphones = Self::headphones();
        let icon = Self::icon(volume, headphones, mute);
        Self { icon, mute, volume }
    }

    fn icon(volume: f32, headphones: bool, mute: bool) -> char {
        if mute {
            return '';
        }
        if headphones {
            return '';
        }
        match volume {
            0.0 => '',
            0.0..=33.0 => '',
            33.0..=100.0 => '',
            _ => '',
        }
    }

    fn headphones() -> bool {
        false
    }

    fn update_wireplumber(&self) -> Result<(), AudioError> {
        let volume = self.volume.clamp(0.0, 100.0) / 100.0;
        Command::new("wpctl")
            .args(["set-volume", "@DEFAULT_SINK@", &format!("{}", volume)])
            .spawn()
            .map_err(|x| CommandError::Command("wpctl set-volume...".to_string(), x.to_string()))
            .map_err(AudioError::Update)
            .map(|_| ())?;
        Command::new("wpctl")
            .args(["set-mute", "@DEFAULT_SINK@", "toggle"])
            .spawn()
            .map_err(|x| CommandError::Command("wpctl set-volume...".to_string(), x.to_string()))
            .map_err(AudioError::Update)
            .map(|_| ())
    }

    fn update_eww(&self) -> Result<(), AudioError> {
        eww_update(EwwVariable::Audio(self.clone())).map_err(AudioError::Update)
    }

    fn update(&self) -> Result<(), AudioError> {
        self.update_eww()?;
        self.update_wireplumber()?;
        Ok(())
    }
}

pub fn set_audio(volume: f32) -> Result<(), AudioError> {
    let settings = AudioSettings::from_volume(volume, false);
    settings.update()
}

type VolumeLevel = f32;
type Mute = bool;
fn get_volume() -> Result<(VolumeLevel, Mute), AudioError> {
    let output = String::from_utf8(
        Command::new("wpctl")
            .args(["get-volume", "@DEFAULT_SINK@"])
            .output()
            .map_err(|x| {
                AudioError::Command(CommandError::Command(
                    "wpctl get-volume".to_string(),
                    x.to_string(),
                ))
            })?
            .stdout,
    )
    .map_err(|x| AudioError::WirePlumber(x.to_string()))?;
    let volume = output.split_whitespace().collect::<Vec<_>>()[1];
    let mute = output.contains("MUTED");
    let volume = volume.parse::<f32>().map_err(|x| {
        AudioError::VolumeParse(ParseError::Volume(volume.to_string(), x.to_string()))
    })?;
    Ok((volume * 100.0, mute))
}

pub fn get_audio() -> Result<(), AudioError> {
    print!(
        "{}",
        serde_json::to_string(&AudioSettings::try_new()?).unwrap()
    );
    Ok(())
}

pub fn increment_volume() -> Result<(), AudioError> {
    let mut settings = AudioSettings::try_new()?;
    settings.volume += 5.0;
    settings.update()
}

pub fn decrement_volume() -> Result<(), AudioError> {
    let mut settings = AudioSettings::try_new()?;
    settings.volume -= 5.0;
    settings.update()
}

pub fn toggle_volume_mute() -> Result<(), AudioError> {
    let mut settings = AudioSettings::try_new()?;
    settings.mute = !settings.mute;
    settings.update()
}

pub fn toggle_mute() -> Result<(), AudioError> {
    Command::new("wpctl")
        .args(["set-mute", "@DEFAULT_SINK@", "toggle"])
        .spawn()
        .map_err(|x| CommandError::Command("wpctl set-volume...".to_string(), x.to_string()))
        .map_err(AudioError::Update)
        .map(|_| ())
}
