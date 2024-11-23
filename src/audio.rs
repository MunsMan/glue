use std::process::Command;

use serde::Serialize;

use crate::error::{AudioError, CommandError, ParseError};
use crate::eww::{eww_update, EwwVariable};

#[derive(Serialize, Clone, Debug, Copy)]
pub enum Mute {
    Active = 0,
    Mute = 1,
}

impl From<bool> for Mute {
    fn from(value: bool) -> Self {
        match value {
            false => Self::Active,
            true => Self::Mute,
        }
    }
}

impl ToString for Mute {
    fn to_string(&self) -> String {
        match self {
            Self::Active => "0".to_string(),
            Self::Mute => "1".to_string(),
        }
    }
}

impl Into<bool> for Mute {
    fn into(self) -> bool {
        match self {
            Self::Active => false,
            Self::Mute => true,
        }
    }
}

impl std::ops::Not for Mute {
    type Output = Mute;

    fn not(self) -> Self::Output {
        match self {
            Self::Mute => Self::Active,
            Self::Active => Self::Mute,
        }
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct AudioSettings {
    volume: f32,
    mute: Mute,
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

    fn icon(volume: f32, headphones: bool, mute: Mute) -> char {
        if mute.into() {
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
            .args(["set-mute", "@DEFAULT_SINK@", self.mute.to_string().as_str()])
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
    let settings = AudioSettings::from_volume(volume, Mute::Active);
    settings.update()
}

type VolumeLevel = f32;
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
    Ok((volume * 100.0, mute.into()))
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
