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
