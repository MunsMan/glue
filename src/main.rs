use tracing::error;

use clap::Parser;
use glue::bin_name;
use hyprland::event_listener::EventListener;

use self::audio::{decrement_volume, get_audio, increment_volume, set_audio, toggle_mute};
use self::battery::get_battery;
use self::cli::{AudioCommand, Cli, Command::*, MicCommand, WorkspaceCommand};
use self::config::Config;
use self::error::{DaemonError, GlueError};
use self::mic::{get_mic, toggle_mic};
use self::start::CommandBuilder;
use self::workspace::{eww_workspace_update, eww_workspaces};

mod audio;
mod battery;
mod cli;
mod config;
mod error;
mod eww;
mod mic;
mod start;
mod workspace;

fn main() {
    let cli = Cli::parse();
    let config = Config::load();
    let result: Result<(), GlueError> = match cli.command {
        Daemon { default_spaces } => daemon(default_spaces).map_err(GlueError::Daemon),
        Workspace {
            default_spaces,
            command,
        } => match command {
            None => eww_workspaces(default_spaces)
                .map_err(GlueError::Workspace)
                .map(|x| {
                    print!("{}", x);
                }),
            Some(WorkspaceCommand::Update { default_spaces }) => {
                eww_workspace_update(default_spaces).map_err(GlueError::Workspace)
            }
        },
        Audio { command } => match command {
            AudioCommand::Set { percent } => set_audio(percent),
            AudioCommand::Get => get_audio(),
            AudioCommand::Mute => toggle_mute(),
            AudioCommand::Increase => increment_volume(),
            AudioCommand::Decrease => decrement_volume(),
        }
        .map_err(GlueError::Audio),
        Mic { command } => match command {
            MicCommand::Mute => toggle_mic(),
            MicCommand::Get => get_mic(),
        }
        .map_err(GlueError::Audio),
        Battery { command } => match command {
            cli::BatteryCommand::Get => get_battery(&config),
        }
        .map_err(GlueError::Battery),
        Start {} => start(),
    };
    if let Err(error) = result {
        error!("{}", error);
    }
}

fn start() -> Result<(), GlueError> {
    let commands = [
        "eww open bar",
        &format!("{} daemon", bin_name()).to_owned(),
        "1password --silent",
    ];
    for command in commands {
        CommandBuilder::try_from(command)
            .map_err(GlueError::Parse)?
            .start()
            .map_err(GlueError::Command)?;
    }
    Ok(())
}

fn daemon(default_spaces: usize) -> Result<(), DaemonError> {
    let mut listener = EventListener::new();
    listener.add_workspace_change_handler(move |_| {
        eww_workspace_update(default_spaces).expect("Unable to update workspace!")
    });
    listener
        .start_listener()
        .map_err(|x| DaemonError::Listener(x.to_string()))
}
