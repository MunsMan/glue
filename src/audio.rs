use std::process::Command;

use serde::Serialize;

use crate::error::{AudioError, CommandError, ParseError};
use crate::eww::{eww_update, EwwVariable};

#[derive(Serialize, Clone, Debug, Copy)]
pub enum SpeakerState {
    Active = 0,
    Mute = 1,
}

impl From<bool> for SpeakerState {
    fn from(value: bool) -> Self {
        match value {
            false => Self::Active,
            true => Self::Mute,
        }
    }
}

impl ToString for SpeakerState {
    fn to_string(&self) -> String {
        match self {
            Self::Active => "0".to_string(),
            Self::Mute => "1".to_string(),
        }
    }
}

impl Into<bool> for SpeakerState {
    fn into(self) -> bool {
        match self {
            Self::Active => false,
            Self::Mute => true,
        }
    }
}

impl std::ops::Not for SpeakerState {
    type Output = SpeakerState;

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
    mute: SpeakerState,
    icon: char,
}

impl AudioSettings {
    fn try_new() -> Result<Self, AudioError> {
        let (volume, mute) = get_volume()?;
        Ok(Self::from_volume(volume, mute))
    }

    fn from_volume(volume: VolumeLevel, mute: SpeakerState) -> Self {
        let headphones = Self::headphones();
        let icon = Self::icon(volume, headphones, mute);
        Self { icon, mute, volume }
    }

    fn icon(volume: f32, headphones: bool, mute: SpeakerState) -> char {
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

    fn toggle_mute(&mut self) -> Result<(), AudioError> {
        self.mute = !self.mute;
        let headphones = Self::headphones();
        self.icon = Self::icon(self.volume, headphones, self.mute);
        Command::new("wpctl")
            .args(["set-mute", "@DEFAULT_SINK@", self.mute.to_string().as_str()])
            .spawn()
            .map_err(|x| CommandError::Command("wpctl set-volume...".to_string(), x.to_string()))
            .map_err(AudioError::Update)
            .map(|_| ())
    }

    fn change_volume(&mut self, volume: f32) -> Result<(), AudioError> {
        self.volume = volume;
        Command::new("wpctl")
            .args(["set-volume", "@DEFAULT_SINK@", &format!("{}", volume)])
            .spawn()
            .map_err(|x| CommandError::Command("wpctl set-volume...".to_string(), x.to_string()))
            .map_err(AudioError::Update)
            .map(|_| ())
    }

    fn update(&self) -> Result<(), AudioError> {
        eww_update(EwwVariable::Audio(self.clone())).map_err(AudioError::Update)?;
        Ok(())
    }
}

pub fn set_audio(volume: f32) -> Result<(), AudioError> {
    let mut settings = AudioSettings::try_new()?;
    settings.change_volume(volume)?;
    settings.update()
}

type VolumeLevel = f32;
fn get_volume() -> Result<(VolumeLevel, SpeakerState), AudioError> {
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
    settings.change_volume(settings.volume + 5.0)?;
    settings.update()
}

pub fn decrement_volume() -> Result<(), AudioError> {
    let mut settings = AudioSettings::try_new()?;
    settings.change_volume(settings.volume - 5.0)?;
    settings.update()
}

pub fn toggle_volume_mute() -> Result<(), AudioError> {
    let mut settings = AudioSettings::try_new()?;
    settings.toggle_mute()?;
    settings.update()
}
