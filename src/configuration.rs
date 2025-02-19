use anyhow::Result;
use config::Config;
use log::LevelFilter;
use std::{path::PathBuf, time::Duration};

use serde::{Deserialize, Serialize};

use crate::error::ConfigurationError;

/// Glue Configuration Definition
/// Defining all user accessable file configuration
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Configuration {
    pub autostart: Vec<String>,
    pub battery: Battery,
    pub battery_path: Option<String>,
    pub coffee: Coffee,
    pub general: General,
    pub hyprland: Hyprland,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Battery {
    pub charging_states: Vec<char>,
    pub full: char,
    pub charging: char,
    pub empty: char,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Coffee {
    pub coffee: char,
    pub relax: char,
    #[serde(with = "humantime_serde")]
    pub notification: Option<Duration>,
}

impl Default for Coffee {
    fn default() -> Self {
        Self {
            coffee: '',
            relax: '󰒲',
            notification: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct General {
    pub log_level: LevelFilter,
    pub eww_config: Option<String>,
}

impl Default for General {
    fn default() -> Self {
        Self {
            log_level: LevelFilter::Info,
            eww_config: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Hyprland {
    pub default_spaces: usize,
}

impl Default for Hyprland {
    fn default() -> Self {
        Self { default_spaces: 5 }
    }
}
