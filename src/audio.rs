use std::process::Command;

use serde::Serialize;

use crate::eww::{eww_update, EwwVariable};

#[derive(Serialize, Clone, Debug)]
pub struct AudioSettings {
    #[serde(skip)]
    headphones: bool,
    volume: f32,
    icon: char,
}

impl AudioSettings {
    fn new() -> Self {
        let volume = get_volume();
        Self::from_volume(volume)
    }

    fn from_volume(volume: f32) -> Self {
        let headphones = Self::headphones();
        let icon = Self::icon(volume, headphones);
        Self {
            headphones,
            icon,
            volume,
        }
    }

    fn icon(volume: f32, headphones: bool) -> char {
        if headphones {
            return '';
        }
        match volume {
            0.0 => '',
            0.0..33.0 => '',
            33.0..100.0 => '',
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
    let settings = AudioSettings::from_volume(volume);
    settings.update();
}

fn get_volume() -> f32 {
    let output = Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_SINK@"])
        .output()
        .map_or_else(
            |_| String::default(),
            |x| String::from_utf8(x.stdout).unwrap(),
        );
    let volume = output.split_whitespace().collect::<Vec<_>>()[1];
    volume.parse::<f32>().unwrap() * 100.0
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
