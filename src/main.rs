use std::env;
use std::error::Error;
use std::ffi::OsString;

use clap::Parser;
use glue::bin_name;
use hyprland::event_listener::EventListener;
use sysinfo::System;

use self::audio::{decrement_volume, get_audio, increment_volume, set_audio, toggle_mute};
use self::battery::get_battery;
use self::cli::{AudioCommand, Cli, Command::*, MicCommand, WorkspaceCommand};
use self::config::Config;
use self::mic::{get_mic, toggle_mic};
use self::start::{start_daemon, CommandBuilder};
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
    match cli.command {
        Daemon { default_spaces } => daemon_starter(default_spaces),
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
        Start {} => {
            let _ = start();
        }
    }
}

fn start() -> Result<(), Box<dyn Error>> {
    start_daemon()?;
    let commands = [
        "eww open bar",
        &format!("{} daemon", bin_name()).to_owned(),
        "1Password --silent",
    ];
    for command in commands {
        CommandBuilder::try_from(command)?.start()?;
    }
    Ok(())
}

fn daemon(default_spaces: usize) {
    let mut listener = EventListener::new();
    listener.add_workspace_change_handler(move |_| eww_workspace_update(default_spaces));
    listener.start_listener().unwrap();
}

fn daemon_starter(default_spaces: usize) {
    let s = System::new_all();
    let name = env::args().next().unwrap();
    let name = name.split("/").last().unwrap();
    if s.processes_by_name(name.as_ref())
        .filter(|x| x.cmd().contains(&OsString::from("daemon")))
        .count()
        > 0
    {
        daemon(default_spaces)
    }
}
