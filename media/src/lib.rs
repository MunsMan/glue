use std::process::Command;

use glue_traits::{FunctionKey, ToggleKey};
use log::error;
use playerctl::{PlayerError, Playerctl};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod cli;
mod playerctl;

pub struct Media {
    config: MediaConfig,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct MediaConfig {
    pub default_player: Option<String>,
}

impl Media {
    fn new(config: Option<MediaConfig>) -> Self {
        Self {
            config: config.unwrap_or(Default::default()),
        }
    }

    fn stop() -> Result<(), MediaError> {
        Playerctl::stop().map_err(|err| MediaError::Playerctl(err))
    }

    fn start(&self) -> Result<(), MediaError> {
        let res = Playerctl::play();
        if let Err(PlayerError::NoPlayer) = res {
            if let Some(player) = &self.config.default_player {
                Command::new(player).spawn().unwrap();
            }
        }
        res.map_err(|err| MediaError::Playerctl(err))
    }

    fn get() -> Result<(), MediaError> {
        let state = Playerctl::get().map_err(|err| MediaError::Playerctl(err))?;
        println!("State: {:#?}", state);
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
        res.map_err(|err| MediaError::Playerctl(err))
    }
}

impl FunctionKey<MediaError> for Media {
    /// Because increase is usally the right key,
    /// it will map to the next/skip button for the media control
    fn increase() -> Result<(), MediaError> {
        Playerctl::next().map_err(|err| MediaError::Playerctl(err))
    }

    /// Because increase is usally the left key,
    /// it will map to the previous/back button for the media control
    fn decrease() -> Result<(), MediaError> {
        Playerctl::previous().map_err(|err| MediaError::Playerctl(err))
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
}
