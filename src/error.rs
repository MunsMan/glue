use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub enum BatteryError {
    UnknownState(String),
    ReadFile(String, String),
    ParseCapacity(String),
}

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

impl Error for BatteryError {}

type Command = String;
type Message = String;

#[derive(Debug)]
pub enum CommandError {
    Pidof,
    AlreadyRunning,
    HyprlandDispatch(Message),
    Command(Command, Message),
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

impl Error for CommandError {}

#[derive(Debug)]
pub enum ParseError {
    Command(String),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Command(input) => {
                write!(f, "Unable to Parse Command Input:\nWrong format: {}", input)
            }
        }
    }
}

impl Error for ParseError {}
