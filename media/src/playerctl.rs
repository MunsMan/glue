use std::process::Command;

use log::info;
use thiserror::Error;

use crate::pest::{parse_metadata, Metadata, MetadataError};

#[derive(Error, Debug)]
pub enum PlayerError {
    #[error("Command Failed: {}", .0)]
    CommandError(String),
    #[error("Unable to toggle player state: {}", .0)]
    Toggle(String),
    #[error("Unable to stop player: {}", .0)]
    Stop(String),
    #[error("No player found")]
    NoPlayer,
    #[error("Parsing Error: {}", .0)]
    ParsingError(String),
    #[error("Parsing Metadata Error: {}", .0)]
    ParsingMetadataError(MetadataError),
}

#[derive(Debug)]
pub enum PlayerState {
    Playing,
    Paused,
}

pub(crate) struct Playerctl {}

impl Playerctl {
    fn command() -> Command {
        Command::new("playerctl")
    }

    fn parse_error(msg: String) -> PlayerError {
        if msg.starts_with("No players found") {
            return PlayerError::NoPlayer;
        }
        PlayerError::CommandError(msg)
    }

    pub(crate) fn get() -> Result<PlayerState, PlayerError> {
        let output = Self::command()
            .arg("status")
            .output()
            .map_err(|err| PlayerError::CommandError(err.to_string()))?;
        if !output.status.success() {
            return Err(Self::parse_error(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }
        let output = String::from_utf8_lossy(&output.stdout).to_string();
        match output.as_str() {
            "Playing\n" => Ok(PlayerState::Playing),
            "Paused\n" => Ok(PlayerState::Paused),
            _ => Err(PlayerError::ParsingError(output)),
        }
    }

    pub(crate) fn stop() -> Result<(), PlayerError> {
        let output = Self::command()
            .arg("pause")
            .output()
            .map_err(|err| PlayerError::CommandError(err.to_string()))?;
        if !output.status.success() {
            return Err(PlayerError::Stop(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }
        Ok(())
    }

    pub(crate) fn play() -> Result<(), PlayerError> {
        let output = Self::command()
            .arg("play")
            .output()
            .map_err(|err| PlayerError::CommandError(err.to_string()))?;
        if !output.status.success() {
            return Err(Self::parse_error(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }
        Ok(())
    }

    pub(crate) fn toggle() -> Result<(), PlayerError> {
        let output = Self::command()
            .arg("play-pause")
            .output()
            .map_err(|err| PlayerError::CommandError(err.to_string()))?;
        if !output.status.success() {
            return Err(PlayerError::Toggle(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }
        Ok(())
    }

    pub(crate) fn next() -> Result<(), PlayerError> {
        let output = Self::command()
            .arg("next")
            .output()
            .map_err(|err| PlayerError::CommandError(err.to_string()))?;
        if !output.status.success() {
            return Err(PlayerError::Toggle(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }
        Ok(())
    }

    pub(crate) fn previous() -> Result<(), PlayerError> {
        let output = Self::command()
            .arg("previous")
            .output()
            .map_err(|err| PlayerError::CommandError(err.to_string()))?;
        info!("Command run!");
        if !output.status.success() {
            return Err(PlayerError::Toggle(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }
        Ok(())
    }

    pub(crate) fn metadata() -> Result<Vec<Metadata>, PlayerError> {
        let output = Self::command()
            .arg("metadata")
            .output()
            .map_err(|err| PlayerError::CommandError(err.to_string()))?;
        if !output.status.success() {
            return Err(PlayerError::Toggle(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }
        parse_metadata(&String::from_utf8_lossy(&output.stdout))
            .map_err(PlayerError::ParsingMetadataError)
    }
}
