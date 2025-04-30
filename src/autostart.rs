use crate::{configuration::Configuration, start::CommandBuilder};
use anyhow::Result;
use log::{error, info};
use notify_rust::Notification;
use std::process::Command;

pub fn auto_start(config: &Configuration) -> Result<()> {
    for program in &config.autostart {
        let command: CommandBuilder = program.as_str().try_into()?;
        let args = command.args.unwrap_or_default();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configuration::Configuration;
    use log::LevelFilter;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn setup() {
        INIT.call_once(|| {
            simplelog::SimpleLogger::init(LevelFilter::Off, simplelog::Config::default()).unwrap();
        });
    }

    #[test]
    fn test_auto_start_empty_config() {
        setup();
        let config = Configuration {
            autostart: vec![],
            ..Default::default()
        };
        let result = auto_start(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_auto_start_single_program() {
        setup();
        let config = Configuration {
            autostart: vec!["echo Hello".to_string()],
            ..Default::default()
        };
        let result = auto_start(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_auto_start_multiple_programs() {
        setup();
        let config = Configuration {
            autostart: vec!["echo Hello".to_string(), "ls -l".to_string()],
            ..Default::default()
        };
        let result = auto_start(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_auto_start_invalid_program() {
        setup();
        let config = Configuration {
            autostart: vec!["nonexistent_program".to_string()],
            ..Default::default()
        };
        let result = auto_start(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_auto_start_mixed_valid_invalid() {
        setup();
        let config = Configuration {
            autostart: vec!["echo Hello".to_string(), "nonexistent_program".to_string()],
            ..Default::default()
        };
        let result = auto_start(&config);
        assert!(result.is_err());
    }
}
