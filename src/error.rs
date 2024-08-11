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
