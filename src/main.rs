use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use commands::Notification;
use key::{FunctionKey, MuteKey};
use serde::{Deserialize, Serialize};
use tracing::error;

use clap::Parser;
use glue::bin_name;
use utils::CancelableTimer;
use wayland::{WaylandClient, WaylandIdle};

use self::audio::{get_audio, set_audio};
use self::battery::get_battery;
use self::cli::{AudioCommand, Cli, Command::*, MicCommand, WorkspaceCommand};
use self::configuration::Configuration;
use self::daemon::daemon;
use self::error::{DaemonError, GlueError};
use self::mic::{get_mic, toggle_mic};
use self::start::run_commands;
use self::workspace::{eww_workspace_update, eww_workspaces};

mod audio;
mod autostart;
mod battery;
mod brightness;
mod cli;
mod coffee;
mod commands;
mod configuration;
mod daemon;
mod error;
mod eww;
mod hyprland;
mod key;
mod mic;
mod monitor;
mod start;
mod utils;
mod wayland;
mod workspace;

pub const GLUE_PATH: &str = "/tmp/glue.sock";

pub(crate) enum Change<T> {
    Add(T),
    Sub(T),
    Absolute(T),
}

fn main() -> Result<()> {
    let mut config = Configuration::load()?;
    let cli = Cli::parse();

    let log_level = match cli.debug {
        0 => log::LevelFilter::Off,
        1 => log::LevelFilter::Error,
        2 => log::LevelFilter::Info,
        3 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };
    simplelog::TermLogger::new(
        log_level,
        simplelog::Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    );
    if cli.debug > 0 {
        config.general.log_level = log_level;
    }
    let result: Result<(), GlueError> = match cli.command {
        Daemon {
            eww_config,
            no_autostart,
        } => daemon(&config, eww_config, no_autostart).map_err(GlueError::Daemon),
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
            AudioCommand::Get => get_audio().map_err(GlueError::Audio),
            AudioCommand::Mute => audio::AudioSettings::mute(),
            AudioCommand::Increase => audio::AudioSettings::increase(),
            AudioCommand::Decrease => audio::AudioSettings::decrease(),
        },
        Mic { command } => match command {
            MicCommand::Mute => toggle_mic(),
            MicCommand::Get => get_mic(),
        }
        .map_err(GlueError::Audio),
        Battery { command } => match command {
            cli::BatteryCommand::Get => match get_battery(&config) {
                Ok(result) => {
                    println!("{}", result);
                    Ok(())
                }
                Err(err) => Err(GlueError::Battery(err)),
            },
        },
        Start {} => start(),
        WakeUp { eww_config } => wake_up(eww_config),
        Lock {} => lock(),
        Coffee { command } => coffee::client(command.into(), &config).map_err(GlueError::Coffee),
        Brightness { command } => match command {
            cli::BrightnessCommand::Get => brightness::BrightnessCtl::get(),
            cli::BrightnessCommand::Increase => brightness::BrightnessCtl::increase(),
            cli::BrightnessCommand::Decrease => brightness::BrightnessCtl::decrease(),
            cli::BrightnessCommand::Set { percent } => brightness::BrightnessCtl::set(percent),
        },
        Test { command } => match command {
            cli::TestCommand::Notification { text } => {
                let res = daemon::client(commands::Command::Notification(Notification::Test(text)));
                match res {
                    Ok(_) => Ok(()),
                    Err(err) => Err(GlueError::DaemonClient(err)),
                }
            }
        },
    };
    if let Err(error) = result {
        error!("{}", error);
        return Err(error.into());
    }
    Ok(())
}

fn start() -> Result<(), GlueError> {
    let commands = ["eww open bar", &format!("{} daemon", bin_name()).to_owned()];
    run_commands(commands.to_vec())
}

fn wake_up(eww_config: Option<String>) -> Result<(), GlueError> {
    eww::open(&eww::WindowName::Bar, eww_config).map_err(GlueError::Command)
}

fn lock() -> Result<(), GlueError> {
    let commands = ["hyprlock"];
    run_commands(commands.to_vec())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IdleState {
    inhibited: bool,
}

impl From<&DaemonState> for IdleState {
    fn from(val: &DaemonState) -> Self {
        IdleState {
            inhibited: val.idle_inhibited,
        }
    }
}

impl From<WaylandIdle> for IdleState {
    fn from(value: WaylandIdle) -> Self {
        IdleState {
            inhibited: value.inhibited,
        }
    }
}

impl From<&WaylandIdle> for IdleState {
    fn from(value: &WaylandIdle) -> Self {
        IdleState {
            inhibited: value.inhibited,
        }
    }
}

#[derive(Clone, Debug)]
struct DaemonState {
    wayland_idle: WaylandClient,
    notification: Option<Duration>,
    idle_notify: Option<CancelableTimer>,
    idle_inhibited: bool,
}

impl DaemonState {
    fn new(config: Arc<Configuration>) -> Result<Self, DaemonError> {
        let wayland_idle = WaylandClient::new().map_err(DaemonError::WaylandError)?;
        Ok(Self {
            wayland_idle,
            idle_notify: None,
            notification: config.coffee.notification,
            idle_inhibited: false,
        })
    }
}
