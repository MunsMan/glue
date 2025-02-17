use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use anyhow::Result;
use audio::toggle_volume_mute;
use autostart::auto_start;
use coffee::{coffeinate, decoffeinate};
use commands::Command;
use glue_ipc::server::Server;
use log::info;
use tracing::error;

use clap::Parser;
use glue::bin_name;
use hyprland::event_listener::EventListener;
use wayland::WaylandClient;

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
mod cli;
mod coffee;
mod commands;
mod configuration;
mod error;
mod eww;
mod mic;
mod start;
mod wayland;
mod workspace;

pub const GLUE_PATH: &str = "/tmp/glue.sock";

fn main() -> Result<()> {
    let cli = Cli::parse();

    let log_level = match cli.debug {
        0 => log::LevelFilter::Off,
        1 => log::LevelFilter::Error,
        2 => log::LevelFilter::Info,
        3 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };
    let _ = simplelog::SimpleLogger::init(log_level, simplelog::Config::default());
    let config = Configuration::load()?;
    let result: Result<(), GlueError> = match cli.command {
        Daemon {
            default_spaces,
            instance,
            eww_config,
        } => daemon(&config, default_spaces, instance, eww_config).map_err(GlueError::Daemon),
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
        WakeUp { eww_config } => wake_up(eww_config),
        Lock {} => lock(),
        Coffee { command } => coffee::client(command.into(), &config).map_err(GlueError::Coffee),
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

fn wake_up(eww_config: Option<String>) -> Result<(), GlueError> {
    eww::open(&eww::WindowName::Bar, eww_config).map_err(GlueError::Command)
}

fn lock() -> Result<(), GlueError> {
    let commands = ["hyprlock; 1password --lock"];
    run_commands(commands.to_vec())
}

#[derive(Clone, Debug)]
struct DaemonState {
    wayland_idle: WaylandClient,
    _hyprland_thread: Arc<Mutex<JoinHandle<Result<(), DaemonError>>>>,
}

impl DaemonState {
    fn new(hyprland_thread: JoinHandle<Result<(), DaemonError>>) -> Result<Self, DaemonError> {
        let wayland_idle = WaylandClient::new().map_err(DaemonError::WaylandError)?;
        let hyprland_thread = Arc::new(Mutex::new(hyprland_thread));
        Ok(Self {
            wayland_idle,
            _hyprland_thread: hyprland_thread,
        })
    }
}

fn daemon(
    config: &Configuration,
    default_spaces: usize,
    instant: Option<String>,
    eww_config: Option<String>,
) -> Result<(), DaemonError> {
    eww::open(&eww::WindowName::Bar, eww_config.clone()).map_err(DaemonError::Command)?;
    auto_start(config).map_err(|x| DaemonError::AutoStart(x))?;

    let thread = std::thread::spawn(move || {
        let mut hyprland_listener = EventListener::new();
        hyprland_listener.add_workspace_changed_handler(move |_| {
            eww_workspace_update(default_spaces).expect("Unable to update workspace!")
        });
        let eww_config_monitor_add = eww_config.clone();
        hyprland_listener.add_monitor_added_handler(move |_| {
            println!("A new Monitor is added!");
            std::thread::sleep(Duration::from_secs(5));
            wake_up(eww_config_monitor_add.clone()).expect("Unable to wake up glue!");
        });
        let eww_config_monitor_remove = eww_config.clone();
        hyprland_listener.add_monitor_removed_handler(move |_| {
            println!("A Monitor is removed!");
            wake_up(eww_config_monitor_remove.clone()).expect("Unable to wake up glue!");
        });
        hyprland_listener
            .start_listener()
            .map_err(|x| DaemonError::Listener(x.to_string()))
    });
    let state = DaemonState::new(thread)?;

    let socket = instant.unwrap_or(GLUE_PATH.to_string());
    let server = Server::new(&socket).map_err(DaemonError::SocketError)?;
    server.listen::<_, Command, _>(
        |command, state, mut stream| {
            match command {
                Command::Coffee(coffee) => match coffee {
                    commands::Coffee::Drink => {
                        info!("Drink Coffee");
                        let result = coffeinate(state);
                        if let Err(err) = result {
                            error!("{}", err);
                        }
                    }
                    commands::Coffee::Relax => {
                        info!("I'm getting sleepy!");
                        let result = decoffeinate(state);
                        if let Err(err) = result {
                            error!("{}", err);
                        }
                    }
                    commands::Coffee::Toggle => {
                        info!("Toggle Coffee State");
                        let result = state.wayland_idle.toggle();
                        if let Err(err) = result {
                            error!("{}", err);
                        }
                    }
                    commands::Coffee::Get => {
                        info!("Coffee Get Request");
                    }
                },
            };
            match state.wayland_idle.get() {
                Ok(state) => {
                    let result = serde_json::to_vec(&state);
                    let Ok(buffer) = result else {
                        error!("Coffee Get: {:#?}", result);
                        return;
                    };
                    stream.write_message(&buffer).unwrap();
                }
                Err(err) => error!("{}", err),
            };
        },
        state,
    );
    Ok(())
}
