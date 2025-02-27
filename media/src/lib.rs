use std::process::Command;

use glue_traits::{FunctionKey, ToggleKey};
use log::error;
use pest::Metadata;
use playerctl::{PlayerError, PlayerState, Playerctl};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod cli;
mod pest;
mod playerctl;

pub struct Media {
    config: MediaConfig,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MediaIcon {
    pub pause: char,
    pub play: char,
}

impl Default for MediaIcon {
    fn default() -> Self {
        Self {
            pause: '',
            play: '',
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MediaConfig {
    pub default_player: Option<String>,
    pub icon: MediaIcon,
}

impl Default for MediaConfig {
    fn default() -> Self {
        Self {
            default_player: None,
            icon: MediaIcon::default(),
        }
    }
}

#[derive(Debug, Serialize)]
struct MediaStatus {
    player: PlayerState,
    metadata: Vec<Metadata>,
    icon: char,
}

impl MediaStatus {
    fn new(player: PlayerState, metadata: Vec<Metadata>, config: &MediaConfig) -> Self {
        let icon = Self::icon(&player, &config);
        Self {
            player,
            metadata,
            icon,
        }
    }

    fn icon(player: &PlayerState, config: &MediaConfig) -> char {
        match player {
            PlayerState::Playing => config.icon.pause,
            PlayerState::Paused => config.icon.play,
        }
    }
}

impl Media {
    fn new(config: Option<MediaConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
        }
    }

    fn stop() -> Result<(), MediaError> {
        Playerctl::stop().map_err(MediaError::Playerctl)
    }

    fn start(&self) -> Result<(), MediaError> {
        let res = Playerctl::play();
        if let Err(PlayerError::NoPlayer) = res {
            if let Some(player) = &self.config.default_player {
                Command::new(player).spawn().unwrap();
            }
        }
        res.map_err(MediaError::Playerctl)
    }

    fn get(&self) -> Result<(), MediaError> {
        let player = Playerctl::get().map_err(MediaError::Playerctl)?;
        let metadata = Playerctl::metadata().map_err(MediaError::Playerctl)?;
        let state = MediaStatus::new(player, metadata, &self.config);
        println!(
            "{}",
            serde_json::to_string(&state).map_err(MediaError::Serialization)?
        );
        Ok(())
    }
}

impl ToggleKey<MediaError> for Media {
    fn toggle(&mut self) -> Result<(), MediaError> {
        let res = Playerctl::toggle();
        if let Err(PlayerError::NoPlayer) = res {
            if let Some(player) = &self.config.default_player {
                Command::new(player).spawn().unwrap();
            }
        }
        res.map_err(MediaError::Playerctl)
    }
}

impl FunctionKey<MediaError> for Media {
    /// Because increase is usally the right key,
    /// it will map to the next/skip button for the media control
    fn increase() -> Result<(), MediaError> {
        Playerctl::next().map_err(MediaError::Playerctl)
    }

    /// Because increase is usally the left key,
    /// it will map to the previous/back button for the media control
    fn decrease() -> Result<(), MediaError> {
        Playerctl::previous().map_err(MediaError::Playerctl)
    }
}

#[derive(Error, Debug)]
pub enum MediaError {
    #[error("There is a player error: {:#?}", .0)]
    Playerctl(PlayerError),
    #[error("Unable to stop media: {:#?}", .0)]
    Stop(String),
    #[error("Unable to attach Observer: {:#?}", .0)]
    Obserever(String),
    #[error("Unable to connect to the dbus: {:#?}", .0)]
    Connection(String),
    #[error("Unable to setup MPRIS Player Finder: {:#?}", .0)]
    PlayerFinderSetup(String),
    #[error("Failed to find players: {:#?}", .0)]
    PlayerFinderFound(String),
    #[error("Unable to Serialize the Status Object: {:#?}", .0)]
    Serialization(serde_json::Error),
}
