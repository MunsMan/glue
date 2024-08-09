use clap::Parser;
use hyprland::event_listener::EventListener;

use self::audio::{decrement_volume, get_audio, increment_volume, set_audio, toggle_mute};
use self::cli::{AudioCommand, Cli, Command as CliCommand, WorkspaceCommand};
use self::workspace::{eww_workspace_update, eww_workspaces};

mod audio;
mod cli;
mod eww;
mod workspace;

fn main() {
    let cli = Cli::parse();
    match cli.command {
        CliCommand::Daemon { default_spaces } => daemon(default_spaces),
        CliCommand::Workspace {
            default_spaces,
            command,
        } => match command {
            None => print!("{}", eww_workspaces(default_spaces)),
            Some(WorkspaceCommand::Update { default_spaces }) => {
                eww_workspace_update(default_spaces)
            }
        },
        CliCommand::Audio { command } => match command {
            AudioCommand::Set { percent } => set_audio(percent),
            AudioCommand::Get => get_audio(),
            AudioCommand::Mute => toggle_mute(),
            AudioCommand::Increase => increment_volume(),
            AudioCommand::Decrease => decrement_volume(),
        },
    }
}

fn daemon(default_spaces: usize) {
    let mut listener = EventListener::new();
    listener.add_workspace_change_handler(move |_| eww_workspace_update(default_spaces));
    listener.start_listener().unwrap();
}
