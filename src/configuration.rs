use anyhow::Result;
use config::Config;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::ConfigurationError;

#[derive(Serialize, Deserialize, Default)]
pub struct Configuration {
    pub battery: Battery,
    pub autostart: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Battery {
    pub charging_states: Vec<char>,
    pub full: char,
    pub charging: char,
    pub empty: char,
}

impl Configuration {
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        let config = Config::builder()
            .add_source(Config::try_from(&Configuration::default())?)
            .add_source(config::File::with_name(&config_path).required(false))
            .add_source(config::Environment::with_prefix("GLUE"))
            .build()?;
        Ok(config.try_deserialize::<Configuration>()?)
    }
    fn get_config_path() -> Result<String> {
        let home = std::env::var("HOME")?;
        let path = PathBuf::from(home)
            .join(".config")
            .join("glue")
            .join("config.toml");
        match path.to_str() {
            Some(s) => Ok(s.to_string()),
            None => Err(ConfigurationError::InvalidPath(path).into()),
        }
    }
}

impl Default for Battery {
    fn default() -> Self {
        Self {
            charging_states: vec!['', '', '', '', ''],
            full: '󱐥',
            charging: '󰂄',
            empty: '',
        }
    }
}
