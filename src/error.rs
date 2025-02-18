use std::path::PathBuf;

use glue_ipc::client::ClientError;
use glue_ipc::server::ServerError;
use hyprland::shared::HyprError;
use thiserror::Error;

use crate::wayland::WaylandClientError;

type Command = String;
type ErrorMessage = String;

#[allow(unused)]
#[derive(Error, Debug)]
pub enum GlueError {
    #[error("{}", .0)]
    Battery(BatteryError),
    #[error("{}", .0)]
    Command(CommandError),
    #[error("{}", .0)]
    Daemon(DaemonError),
    #[error("{}", .0)]
    Parse(ParseError),
    #[error("{}", .0)]
    Audio(AudioError),
    #[error("{}", .0)]
    Workspace(WorkspaceError),
    #[error("{}", .0)]
    Coffee(CoffeeError),
    #[error("{}", .0)]
    Brightness(BrightnessError),
}

#[derive(Error, Debug)]
pub enum BrightnessError {
    #[error("{}", .0)]
    Brightness(brightness::Error),
    #[error("{}", .0)]
    Serialization(String),
}

#[derive(Error, Debug)]
pub enum AudioError {
    #[error("{}", .0)]
    Command(CommandError),
    #[error("{}", .0)]
    Update(CommandError),
    #[error("Unable to parse Volume\n{}", .0)]
    VolumeParse(ParseError),
    #[error("Unable to query Wireplumber: {}", .0)]
    WirePlumber(ErrorMessage),
    #[error("Volume is higher then 100% - Current Volume: {}", .0)]
    VolumeSetting(f32),
}

#[derive(Debug, Error)]
pub enum BatteryError {
    #[error("Unknown Battery State: {}", .0)]
    UnknownState(String),
    #[error("Unable to read file: {}\nOS: {}", .0, .0)]
    ReadFile(String, String),
    #[error("Unable to parse {} as u8 representing the battery level (in %)", .0)]
    ParseCapacity(String),
}

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("Unable to run `pidof` command!")]
    Pidof,
    #[error("Unable to start, because it is already running")]
    AlreadyRunning,
    #[error("Unable to execute {}", .0)]
    HyprlandDispatch(ErrorMessage),
    #[error("Unable to execute {}:\n{}", .0, .1)]
    Command(Command, ErrorMessage),
}

#[derive(Debug, Error)]
pub enum DaemonError {
    #[error("Unable to start Listening...\nERROR: {}", .0)]
    Listener(ErrorMessage),
    #[error("{:#?}", .0)]
    Command(CommandError),
    #[error("Auto Start Error: {:#?}", .0)]
    AutoStart(anyhow::Error),
    #[error("Something with the Socket went wrong: {}", .0)]
    SocketError(ServerError),
    #[error("Wayland error in the Daemon: {}", .0)]
    WaylandError(WaylandClientError),
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Unable to Parse Command Input:\nWrong format: {}", .0)]
    Command(String),
    #[error("Unable to Parse Volume Level:\nInput:{}\nError: {}", .0, .1)]
    Volume(String, ErrorMessage),
}

#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("Unable to get Workspaces from Hyprland\nERROR: {}", .0)]
    Hyprland(HyprError),
    #[error("Unable to update Workspace, recieved: {}", .0)]
    Command(CommandError),
}

#[allow(unused)]
#[derive(Debug, Error)]
pub enum ConfigurationError {
    #[error("Invalid Path found: {:?}", .0)]
    InvalidPath(PathBuf),
}

#[derive(Debug, Error)]
pub enum CoffeeError {
    #[error("Unable to reach the daemon{}", .0)]
    IPCError(ClientError),
    #[error("Something with Wayland didn't work: {}", .0)]
    WaylandError(WaylandClientError),
}

impl Error for AudioError {}
impl Error for BatteryError {}
impl Error for CommandError {}
impl Error for GlueError {}
impl Error for ParseError {}
impl Error for WorkspaceError {}

impl Display for BatteryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BatteryError::UnknownState(state) => write!(f, "Unknown Battery State: {}", state),
            BatteryError::ReadFile(filename, hint) => {
                write!(f, "Unable to read file: {}\nOS: {}", filename, hint)
            }
            BatteryError::ParseCapacity(capacity) => write!(
                f,
                "Unable to parse {} as u8 representing the battery level (in %)",
                capacity
            ),
        }
    }
}

impl Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            CommandError::Pidof => "Unable to run `pidof` command!",
            CommandError::AlreadyRunning => "Unable to start, because it is already running",
            CommandError::HyprlandDispatch(message) => &format!("Unable to execute {}", message),
            CommandError::Command(command, message) => {
                &format!("Unable to execute {}:\n{}", command, message)
            }
        };
        write!(f, "{}", output)
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Command(input) => {
                write!(f, "Unable to Parse Command Input:\nWrong format: {}", input)
            }
            ParseError::Volume(input, error) => {
                write!(
                    f,
                    "Unable to Parse Volume Level:\nInput:{}\nError: {}",
                    input, error
                )
            }
        }
    }
}

impl Display for WorkspaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceError::Hyprland(error_message) => {
                write!(
                    f,
                    "Unable to get Workspaces from Hyprland\nERROR: {}",
                    error_message
                )
            }
            WorkspaceError::Command(command) => {
                write!(f, "Unable to update Workspace, recieved: {}", command)
            }
        }
    }
}

impl Display for GlueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = self;
        write!(f, "{}", x)
    }
}

impl Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioError::WirePlumber(err) => write!(f, "Unable to query Wireplumber: {}", err),
            AudioError::VolumeParse(err) => write!(f, "Unable to parse Volume\n{}", err),
            AudioError::Command(err) => write!(f, "{}", err),
            AudioError::Update(err) => write!(f, "{}", err),
            AudioError::VolumeSetting(err) => write!(f, "{}", err),
        }
    }
}
