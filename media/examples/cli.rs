use clap::Parser;
use std::error::Error;

use glue_media::{
    cli::{self, Cli},
    MediaConfig,
};

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let config = MediaConfig {
        default_player: Some("zen".to_string()),
    };
    match cli.command {
        cli::Command::Media(subcommand) => cli::handler(subcommand, Some(config)),
    }?;
    Ok(())
}
