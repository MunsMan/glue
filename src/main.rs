use std::time::Duration;

use anyhow::Result;
use audio::toggle_volume_mute;
use autostart::auto_start;
use key::FunctionKey;
use tracing::error;

use clap::Parser;
use glue::bin_name;
use hyprland::event_listener::EventListener;

use self::audio::{decrement_volume, get_audio, increment_volume, set_audio};
use self::battery::get_battery;
use self::cli::{AudioCommand, Cli, Command::*, MicCommand, WorkspaceCommand};
use self::configuration::Configuration;
use self::error::{DaemonError, GlueError};
use self::mic::{get_mic, toggle_mic};
use self::start::run_commands;
use self::workspace::{eww_workspace_update, eww_workspaces};

mod audio;
mod autostart;
mod battery;
mod brightness;
mod cli;
mod configuration;
mod error;
mod eww;
mod key;
mod mic;
mod start;
mod workspace;

pub(crate) enum Change<T> {
    Add(T),
    Sub(T),
    Absolute(T),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let log_level = match cli.debug {
        0 => log::LevelFilter::Off,
        1 => log::LevelFilter::Error,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Info,
    };
    let _ = simplelog::SimpleLogger::init(log_level, simplelog::Config::default());
    let config = Configuration::load()?;
    let result: Result<(), GlueError> = match cli.command {
        Daemon { default_spaces } => daemon(&config, default_spaces).map_err(GlueError::Daemon),
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
        Brightness { command } => match command {
            cli::BrightnessCommand::Get => brightness::BrightnessCtl::get(),
            cli::BrightnessCommand::Increase => brightness::BrightnessCtl::increase(),
            cli::BrightnessCommand::Decrease => brightness::BrightnessCtl::decrease(),
            cli::BrightnessCommand::Set { percent } => brightness::BrightnessCtl::set(percent),
        },
    };
    if let Err(error) = result {
        error!("{}", error);
        return Err(error.into());
    }
    Ok(())
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

fn daemon(config: &Configuration, default_spaces: usize) -> Result<(), DaemonError> {
    eww::open(&eww::WindowName::Bar).map_err(DaemonError::Command)?;
    auto_start(config).map_err(|x| DaemonError::AutoStart(x))?;
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
