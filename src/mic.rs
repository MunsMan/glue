use std::process::Command;

use serde::Serialize;

use crate::error::{AudioError, CommandError, ParseError};
use crate::eww::{eww_update, EwwVariable};

#[derive(Clone, Debug)]
pub enum MicState {
    Muted,
    Unmuted,
}

impl MicState {
    fn toggle(&self) -> Self {
        match self {
            MicState::Muted => Self::Unmuted,
            MicState::Unmuted => Self::Muted,
        }
    }

    fn icon(&self) -> char {
        match self {
            MicState::Muted => '',
            MicState::Unmuted => '',
        }
    }
}

impl Serialize for MicState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_char(self.icon())
    }
}

impl From<&MicState> for &'static str {
    fn from(state: &MicState) -> Self {
        match state {
            MicState::Muted => "1",
            MicState::Unmuted => "0",
        }
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct MicSettings {
    volume: f32,
    state: MicState,
}

impl MicSettings {
    fn try_new() -> Result<Self, AudioError> {
        let (volume, mute) = get_mic_info()?;
        Ok(Self {
            volume,
            state: mute,
        })
    }

    fn icon(mute: &MicState) -> char {
        match mute {
            MicState::Muted => '',
            MicState::Unmuted => '',
        }
    }

    fn update(&self) -> Result<(), CommandError> {
        eww_update(EwwVariable::Mic(self.clone()))
    }

    fn toggle_mute(&mut self) -> Result<(), AudioError> {
        let _ = Command::new("wpctl")
            .args(["set-mute", "@DEFAULT_SOURCE@", (&self.state).into()])
            .spawn()
            .map_err(|x| CommandError::Command("wpctl set-mute...".to_string(), x.to_string()))
            .map_err(AudioError::Update);
        self.state = self.state.toggle();
        Ok(())
    }
}

type MicVolumeLevel = f32;

fn get_mic_info() -> Result<(MicVolumeLevel, MicState), AudioError> {
    let output = Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_SOURCE@"])
        .output()
        .map_or_else(
            |_| String::default(),
            |x| String::from_utf8(x.stdout).unwrap(),
        );
    let volume = output.split_whitespace().collect::<Vec<_>>()[1];
    let mute = if output.contains("MUTED") {
        MicState::Muted
    } else {
        MicState::Unmuted
    };
    let volume = volume
        .parse::<f32>()
        .map_err(|x| ParseError::Volume(volume.to_string(), x.to_string()))
        .map_err(AudioError::VolumeParse)?;
    Ok((volume * 100.0, mute))
}

pub fn get_mic() -> Result<(), AudioError> {
    let settings = MicSettings::try_new()?;
    print!("{}", serde_json::to_string(&settings).unwrap());
    // settings.update().map_err(AudioError::Command)
    Ok(())
}

pub fn toggle_mic() -> Result<(), AudioError> {
    let mut settings = MicSettings::try_new()?;
    dbg!(&settings);
    settings.toggle_mute()?;
    dbg!(&settings);
    println!("{}", serde_json::to_string(&settings).unwrap());
    settings.update().map_err(AudioError::Command)
}
