use hyprland::dispatch::{Dispatch, DispatchType};
use std::process::Command;
use system_utils::bin_name;

use crate::error::{CommandError, ParseError};

pub struct CommandBuilder {
    name: String,
    args: Option<Vec<String>>,
    allow_dup: bool,
}

impl CommandBuilder {
    pub fn new(command: &str) -> Self {
        Self {
            name: command.to_string(),
            args: None,
            allow_dup: false,
        }
    }

    pub fn arg(mut self, arg: &str) -> Self {
        if let Some(ref mut args) = self.args {
            args.push(arg.to_string());
        } else {
            self.args = Some(vec![arg.to_string()]);
        }
        self
    }

    pub fn args(mut self, args: Vec<&str>) -> Self {
        if let Some(ref mut current_args) = self.args {
            for arg in args {
                current_args.push(arg.to_owned())
            }
        } else {
            self.args = Some(
                args.into_iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>(),
            );
        }
        self
    }

    pub fn allow_duplicates(mut self, state: bool) -> Self {
        self.allow_dup = state;
        self
    }

    pub fn command(&self) -> String {
        let mut command = self.name.to_string();
        if let Some(args) = &self.args {
            for arg in args {
                command.push_str(arg);
            }
        }
        command
    }

    pub fn start(&self) -> Result<(), CommandError> {
        start_program(self)
    }
}

impl TryFrom<&str> for CommandBuilder {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut iter = value.split_whitespace();
        let name = iter.next().ok_or(ParseError::Command(value.to_string()))?;
        let args = iter.collect::<Vec<&str>>();
        Ok(CommandBuilder::new(name).args(args))
    }
}

fn try_already_running(command: &CommandBuilder) -> Result<bool, CommandError> {
    let output = Command::new("pidof")
        .arg(&command.name)
        .output()
        .map_err(|_| CommandError::Pidof)?;
    Ok(!output.stdout.is_empty())
}

fn hyprrun(command: &CommandBuilder) -> Result<(), CommandError> {
    Dispatch::call(DispatchType::Exec(&command.command()))
        .map_err(|x| CommandError::HyprlandDispatch(x.to_string()))?;
    Ok(())
}

fn start_program(command: &CommandBuilder) -> Result<(), CommandError> {
    let running = try_already_running(command)?;
    if !running || command.allow_dup {
        let name = command.name.clone();
        let mut builder = Command::new(name.clone());
        if let Some(args) = &command.args {
            for arg in args {
                builder.arg(arg);
            }
        };
        hyprrun(command)?;
        Ok(())
    } else {
        Err(CommandError::AlreadyRunning)
    }
}

pub fn start_daemon() -> Result<(), CommandError> {
    Dispatch::call(DispatchType::Exec(
        format!("{} daemon", bin_name()).as_str(),
    ))
    .map_err(|x| CommandError::HyprlandDispatch(x.to_string()))?;
    Ok(())
}
