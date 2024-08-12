use std::process::Command;

use serde::Serialize;

use crate::eww::{eww_update, EwwVariable};

#[derive(Serialize, Clone)]
pub struct MicSettings {
    volume: f32,
    mute: bool,
    icon: char,
}

impl MicSettings {
    fn new() -> Self {
        let (volume, mute) = get_mic_info();
        let icon = Self::icon(mute);
        Self { volume, mute, icon }
    }

    fn icon(mute: Mute) -> char {
        match mute {
            true => '',
            false => '',
        }
    }

    fn update(&self) {
        eww_update(EwwVariable::Mic(self.clone())).unwrap();
    }
}

type MicVolumeLevel = f32;
type Mute = bool;

fn get_mic_info() -> (MicVolumeLevel, Mute) {
    let output = Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_SOURCE@"])
        .output()
        .map_or_else(
            |_| String::default(),
            |x| String::from_utf8(x.stdout).unwrap(),
        );
    let volume = output.split_whitespace().collect::<Vec<_>>()[1];
    let mute = output.contains("MUTED");
    (volume.parse::<f32>().unwrap() * 100.0, mute)
}

pub fn get_mic() {
    let settings = MicSettings::new();
    settings.update();
}

pub fn toggle_mic() {
    Command::new("wpctl")
        .args(["set-mute", "@DEFAULT_SOURCE@", "toggle"])
        .spawn()
        .unwrap();
    let settings = MicSettings::new();
    settings.update();
}
