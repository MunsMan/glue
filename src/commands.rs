use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Command {
    Coffee(Coffee),
}

#[derive(Serialize, Deserialize)]
pub enum Coffee {
    Drink,
    Relex,
}

impl Into<Command> for Coffee {
    fn into(self) -> Command {
        Command::Coffee(self)
    }
}
