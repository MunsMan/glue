use std::time::Duration;

use audio::toggle_volume_mute;
use tracing::error;

use clap::Parser;
use glue::bin_name;
use hyprland::event_listener::EventListener;

use self::audio::{decrement_volume, get_audio, increment_volume, set_audio};
use self::battery::get_battery;
use self::cli::{AudioCommand, Cli, Command::*, MicCommand, WorkspaceCommand};
use self::config::Config;
use self::error::{DaemonError, GlueError};
use self::mic::{get_mic, toggle_mic};
use self::start::run_commands;
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
            AudioCommand::Mute => toggle_volume_mute(),
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
        WakeUp {} => wake_up(),
        Lock {} => lock(),
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
    run_commands(commands.to_vec())
}

fn wake_up() -> Result<(), GlueError> {
    eww::open(&eww::WindowName::Bar).map_err(GlueError::Command)
}

fn lock() -> Result<(), GlueError> {
    let commands = ["hyprlock; 1password --lock"];
    run_commands(commands.to_vec())
}

fn daemon(default_spaces: usize) -> Result<(), DaemonError> {
    eww::open(&eww::WindowName::Bar).map_err(DaemonError::Command)?;
    let mut listener = EventListener::new();
    listener.add_workspace_changed_handler(move |_| {
        eww_workspace_update(default_spaces).expect("Unable to update workspace!")
    });
    listener.add_monitor_added_handler(move |_| {
        println!("A new Monitor is added!");
        std::thread::sleep(Duration::from_secs(5));
        wake_up().expect("Unable to wake up glue!");
    });
    listener.add_monitor_removed_handler(move |_| {
        println!("A Monitor is removed!");
        wake_up().expect("Unable to wake up glue!");
    });
    listener
        .start_listener()
        .map_err(|x| DaemonError::Listener(x.to_string()))
}
