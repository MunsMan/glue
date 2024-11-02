use std::error::Error;
use std::fmt::Display;

use hyprland::shared::HyprError;

type Command = String;
type ErrorMessage = String;

#[allow(unused)]
#[derive(Debug)]
pub enum GlueError {
    Battery(BatteryError),
    Command(CommandError),
    Daemon(DaemonError),
    Parse(ParseError),
    Audio(AudioError),
    Workspace(WorkspaceError),
}

#[derive(Debug)]
pub enum AudioError {
    Command(CommandError),
    Update(CommandError),
    VolumeParse(ParseError),
    WirePlumber(ErrorMessage),
}

#[derive(Debug)]
pub enum BatteryError {
    UnknownState(String),
    ReadFile(String, String),
    ParseCapacity(String),
}

#[derive(Debug)]
pub enum CommandError {
    Pidof,
    AlreadyRunning,
    HyprlandDispatch(ErrorMessage),
    Command(Command, ErrorMessage),
}

#[derive(Debug)]
pub enum DaemonError {
    Listener(ErrorMessage),
    Command(CommandError),
}

#[derive(Debug)]
pub enum ParseError {
    Command(String),
    Volume(String, ErrorMessage),
}

#[derive(Debug)]
pub enum WorkspaceError {
    Hyprland(HyprError),
    Command(CommandError),
}

impl Error for AudioError {}
impl Error for BatteryError {}
impl Error for CommandError {}
impl Error for DaemonError {}
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

impl Display for DaemonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DaemonError::Listener(error_message) => {
                write!(f, "Unable to start Listening...\nERROR: {}", error_message)
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
        }
    }
}
