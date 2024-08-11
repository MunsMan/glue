use clap::Parser;
use hyprland::event_listener::EventListener;

use self::audio::{decrement_volume, get_audio, increment_volume, set_audio, toggle_mute};
use self::battery::get_battery;
use self::cli::{AudioCommand, Cli, Command::*, MicCommand, WorkspaceCommand};
use self::config::Config;
use self::mic::{get_mic, toggle_mic};
use self::workspace::{eww_workspace_update, eww_workspaces};

mod audio;
mod battery;
mod cli;
mod config;
mod error;
mod eww;
mod mic;
mod workspace;

fn main() {
    let cli = Cli::parse();
    let config = Config::load();
    match cli.command {
        Daemon { default_spaces } => daemon(default_spaces),
        Workspace {
            default_spaces,
            command,
        } => match command {
            None => print!("{}", eww_workspaces(default_spaces)),
            Some(WorkspaceCommand::Update { default_spaces }) => {
                eww_workspace_update(default_spaces)
            }
        },
        Audio { command } => match command {
            AudioCommand::Set { percent } => set_audio(percent),
            AudioCommand::Get => get_audio(),
            AudioCommand::Mute => toggle_mute(),
            AudioCommand::Increase => increment_volume(),
            AudioCommand::Decrease => decrement_volume(),
        },
        Mic { command } => match command {
            MicCommand::Mute => toggle_mic(),
            MicCommand::Get => get_mic(),
        },
        Battery { command } => match command {
            cli::BatteryCommand::Get => {
                let _ = get_battery(&config);
            }
        },
    }
}

fn daemon(default_spaces: usize) {
    let mut listener = EventListener::new();
    listener.add_workspace_change_handler(move |_| eww_workspace_update(default_spaces));
    listener.start_listener().unwrap();
}
