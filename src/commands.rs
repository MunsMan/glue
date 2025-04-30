use serde::{Deserialize, Serialize};

use crate::cli;

#[derive(Serialize, Deserialize)]
pub enum Command {
    Coffee(Coffee),
}

#[derive(Serialize, Deserialize)]
pub enum Coffee {
    Drink,
    Relax,
    Toggle,
    Get,
}

impl From<Coffee> for Command {
    fn from(val: Coffee) -> Self {
        Command::Coffee(val)
    }
}

impl From<cli::CoffeeCommand> for Coffee {
    fn from(value: cli::CoffeeCommand) -> Self {
        match value {
            cli::CoffeeCommand::Drink => Self::Drink,
            cli::CoffeeCommand::Relax => Self::Relax,
            cli::CoffeeCommand::Toggle => Self::Toggle,
            cli::CoffeeCommand::Get => Self::Get,
        }
    }
}
