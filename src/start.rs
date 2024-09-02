use hyprland::dispatch::{Dispatch, DispatchType};
use std::process::{self, Command};

use crate::error::{CommandError, GlueError, ParseError};

#[derive(Debug)]
pub struct CommandBuilder {
    name: String,
    args: Option<Vec<String>>,
}

impl CommandBuilder {
    pub fn new(command: &str) -> Self {
        Self {
            name: command.to_string(),
            args: None,
        }
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

    pub fn command(&self) -> String {
        let mut command = self.name.to_string();
        if let Some(args) = &self.args {
            for arg in args {
                command.push(' ');
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
    if output.stdout.is_empty() {
        return Ok(false);
    }
    let pid = process::id();
    let pids = String::from_utf8(output.stdout)
        .map_err(|_| CommandError::Pidof)?
        .split_whitespace()
        .flat_map(|x| x.parse::<u32>().ok())
        .filter(|x| *x != pid)
        .collect::<Vec<u32>>();
    Ok(!pids.is_empty())
}

fn hyprrun(command: &CommandBuilder) -> Result<(), CommandError> {
    Dispatch::call(DispatchType::Exec(&command.command()))
        .map_err(|x| CommandError::HyprlandDispatch(x.to_string()))?;
    Ok(())
}

fn start_program(command: &CommandBuilder) -> Result<(), CommandError> {
    let running = try_already_running(command)?;
    if !running {
        hyprrun(command)?;
        Ok(())
    } else {
        Err(CommandError::AlreadyRunning)
    }
}

pub fn run_commands(commands: Vec<&str>) -> Result<(), GlueError> {
    for command in commands {
        CommandBuilder::try_from(command)
            .map_err(GlueError::Parse)?
            .start()
            .map_err(GlueError::Command)?;
    }
    Ok(())
}
