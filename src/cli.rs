use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Daemon {
        #[arg(default_value_t = 5)]
        default_spaces: usize,
    },
    Workspace {
        #[arg(default_value_t = 5)]
        default_spaces: usize,
        #[command(subcommand)]
        command: Option<WorkspaceCommand>,
    },
    Audio {
        #[command(subcommand)]
        command: AudioCommand,
    },
}

#[derive(Subcommand)]
pub enum WorkspaceCommand {
    Update {
        #[arg(default_value_t = 5)]
        default_spaces: usize,
    },
}

#[derive(Subcommand)]
pub enum AudioCommand {
    Set { percent: f32 },
    Get,
    Mute,
    Increase,
    Decrease,
}