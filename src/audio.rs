use std::cmp::min;
use std::process::Command;

use serde::Serialize;

use crate::error::{AudioError, CommandError, GlueError, ParseError};
use crate::eww::{eww_update, EwwVariable};
use crate::key::{Changeable, FunctionKey, MuteKey};
use crate::Change;

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
    volume: u8,
    mute: SpeakerState,
    icon: char,
}

impl FunctionKey for AudioSettings {
    fn increase() -> Result<(), GlueError> {
        let mut audio = Self::try_new().map_err(GlueError::Audio)?;
        audio.change(Change::Add(5))
    }

    fn decrease() -> Result<(), crate::error::GlueError> {
        let mut audio = Self::try_new().map_err(GlueError::Audio)?;
        audio.change(Change::Sub(5))
    }
}

impl MuteKey for AudioSettings {
    fn mute() -> Result<(), GlueError> {
        let mut audio = AudioSettings::try_new().map_err(GlueError::Audio)?;
        audio.toggle_mute().map_err(GlueError::Audio)?;
        audio.update().map_err(GlueError::Audio)
    }
}

impl Changeable<u8> for AudioSettings {
    fn change(&mut self, change: Change<u8>) -> Result<(), GlueError> {
        match change {
            Change::Add(value) => self.volume = min(self.volume + value, 100),
            Change::Sub(value) => self.volume = min(self.volume - value, 100),
            Change::Absolute(value) => self.volume = min(value, 100),
        }
        Command::new("wpctl")
            .args([
                "set-volume",
                "@DEFAULT_SINK@",
                &format!("{:.2}", self.volume as f32 / 100.0),
            ])
            .spawn()
            .map_err(|x| CommandError::Command("wpctl set-volume...".to_string(), x.to_string()))
            .map_err(AudioError::Update)
            .map_err(GlueError::Audio)?;
        self.update().map_err(GlueError::Audio)
    }
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

    fn icon(volume: u8, headphones: bool, mute: SpeakerState) -> char {
        if mute.into() {
            return '';
        }
        if headphones {
            return '';
        }
        match volume {
            0 => '',
            0..=33 => '',
            34..=100 => '',
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

    fn update(&self) -> Result<(), AudioError> {
        eww_update(EwwVariable::Audio(self.clone())).map_err(AudioError::Update)?;
        Ok(())
    }
}

pub fn set_audio(volume: f32) -> Result<(), GlueError> {
    let volume = min(volume.floor() as u8, 100);
    let mut audio = AudioSettings::try_new().map_err(GlueError::Audio)?;
    audio.change(Change::Absolute(volume))
}

type VolumeLevel = u8;
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
    let volume = min((volume * 100.0).floor() as u8, 100);
    Ok((volume, mute.into()))
}

pub fn get_audio() -> Result<(), AudioError> {
    print!(
        "{}",
        serde_json::to_string(&AudioSettings::try_new()?).unwrap()
    );
    Ok(())
}
