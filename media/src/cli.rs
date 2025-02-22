use clap::{Parser, Subcommand};
use glue_traits::{FunctionKey, ToggleKey};

use crate::{Media, MediaConfig, MediaError};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(subcommand)]
    Media(MediaSubcommand),
}

#[derive(Subcommand)]
pub enum MediaSubcommand {
    Toggle,
    Pause,
    Play,
    Next,
    Previous,
    Status,
}

pub fn handler(subcommand: MediaSubcommand, config: Option<MediaConfig>) -> Result<(), MediaError> {
    match subcommand {
        MediaSubcommand::Toggle => Media::new(config).toggle(),
        MediaSubcommand::Pause => Media::stop(),
        MediaSubcommand::Play => Media::new(config).start(),
        MediaSubcommand::Next => Media::increase(),
        MediaSubcommand::Previous => Media::decrease(),
        MediaSubcommand::Status => Media::get(),
    }
}
