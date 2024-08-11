use std::process::Command;

use serde::Serialize;

use crate::eww::{eww_update, EwwVariable};

#[derive(Serialize, Clone, Debug)]
pub struct AudioSettings {
    #[serde(skip)]
    headphones: bool,
    volume: f32,
    mute: bool,
    icon: char,
}

impl AudioSettings {
    fn new() -> Self {
        let (volume, mute) = get_volume();
        Self::from_volume(volume, mute)
    }

    fn from_volume(volume: VolumeLevel, mute: Mute) -> Self {
        let headphones = Self::headphones();
        let icon = Self::icon(volume, headphones, mute);
        Self {
            headphones,
            icon,
            mute,
            volume,
        }
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

    fn update_wireplumber(&self) {
        let volume = self.volume.clamp(0.0, 100.0) / 100.0;
        Command::new("wpctl")
            .args(["set-volume", "@DEFAULT_SINK@", &format!("{}", volume)])
            .spawn()
            .unwrap();
    }

    fn update_eww(&self) {
        let _ = eww_update(EwwVariable::Audio(self.clone()));
    }

    fn update(&self) {
        self.update_eww();
        self.update_wireplumber();
    }
}

pub fn set_audio(volume: f32) {
    let settings = AudioSettings::from_volume(volume, false);
    settings.update();
}

type VolumeLevel = f32;
type Mute = bool;
fn get_volume() -> (VolumeLevel, Mute) {
    let output = Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_SINK@"])
        .output()
        .map_or_else(
            |_| String::default(),
            |x| String::from_utf8(x.stdout).unwrap(),
        );
    let volume = output.split_whitespace().collect::<Vec<_>>()[1];
    let mute = output.contains("MUTED");
    (volume.parse::<f32>().unwrap() * 100.0, mute)
}

pub fn get_audio() {
    print!("{}", serde_json::to_string(&AudioSettings::new()).unwrap())
}

pub fn increment_volume() {
    let mut settings = AudioSettings::new();
    settings.volume += 5.0;
    settings.update();
}

pub fn decrement_volume() {
    let mut settings = AudioSettings::new();
    settings.volume -= 5.0;
    settings.update();
}

pub fn toggle_mute() {
    Command::new("wpctl")
        .args(["set-mute", "@DEFAULT_SINK@", "toggle"])
        .spawn()
        .unwrap();
}
