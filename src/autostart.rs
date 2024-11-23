use crate::{configuration::Configuration, start::CommandBuilder};
use anyhow::Result;
use log::{error, info};
use notify_rust::Notification;
use std::process::Command;

pub fn auto_start(config: &Configuration) -> Result<()> {
    for program in &config.autostart {
        let command: CommandBuilder = program.as_str().try_into()?;
        let args = command.args.unwrap_or(Vec::new());
        match Command::new(command.name).args(args).spawn() {
            Ok(_) => info!("autostart successful: {}", &program),
            Err(err) => {
                error!("autostart failed: {}\n{:#?}", &program, err);
                Notification::new().summary(&format!("Glue failed to autostart {}", &program));
                return Err(err.into());
            }
        }
    }
    Ok(())
}
