use clap::Parser;
use hyprland::event_listener::EventListener;

use self::audio::set_audio;
use self::cli::{AudioCommand, Cli, Command as CliCommand, WorkspaceCommand};
use self::eww::{eww_workspace_update, eww_workspaces};

mod audio;
mod cli;
mod eww;

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
        },
    }
}

fn daemon(default_spaces: usize) {
    let mut listener = EventListener::new();
    listener.add_workspace_change_handler(move |_| eww_workspace_update(default_spaces));
    listener.start_listener().unwrap();
}
