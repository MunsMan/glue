use std::fmt::Display;
use std::fs;
use std::io::Read;
use std::path::Path;

use serde::Serialize;

use crate::config::Config;
use crate::error::BatteryError;

const BASE_DIR: &str = "/sys/class/power_supply";

#[derive(Serialize)]
pub struct Battery {
    state: BatteryState,
    capacity: u8,
    icon: char,
}

#[derive(Serialize)]
enum BatteryState {
    Charging,
    Discharging,
    Empty,
    Full,
    NotCharging,
}

impl TryFrom<&str> for BatteryState {
    type Error = BatteryError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Charging" => Ok(BatteryState::Charging),
            "Discharging" => Ok(BatteryState::Discharging),
            "Empty" => Ok(BatteryState::Empty),
            "Full" => Ok(BatteryState::Full),
            "Not charging" => Ok(BatteryState::NotCharging),
            x => Err(BatteryError::UnknownState(x.to_string())),
        }
    }
}

impl From<&BatteryState> for String {
    fn from(val: &BatteryState) -> Self {
        match val {
            BatteryState::Charging => "Charging".to_string(),
            BatteryState::Discharging => "Discharging".to_string(),
            BatteryState::Empty => "Empty".to_string(),
            BatteryState::Full => "Full".to_string(),
            BatteryState::NotCharging => "Not charging".to_string(),
        }
    }
}

impl Display for BatteryState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Into::<String>::into(self))
    }
}

impl Battery {
    fn try_new(config: &Config) -> Result<Self, BatteryError> {
        let state = Self::read_state()?;
        let capacity = Self::read_capacity()?;
        let icon = Self::icon(&state, capacity, config);
        Ok(Self {
            state,
            capacity,
            icon,
        })
    }

    fn icon(state: &BatteryState, capacity: u8, config: &Config) -> char {
        match state {
            BatteryState::NotCharging => config.battery.full,
            BatteryState::Full => config.battery.full,
            BatteryState::Discharging => {
                let icons = &config.battery.charging_states;
                let index = capacity / (100 / icons.len() as u8);
                *icons.get(index as usize).unwrap_or(icons.last().unwrap())
            }
            BatteryState::Charging => config.battery.charging,
            BatteryState::Empty => config.battery.empty,
        }
    }

    fn read_state() -> Result<BatteryState, BatteryError> {
        Self::read_sys_file("status")?.trim_end().try_into()
    }

    fn read_capacity() -> Result<u8, BatteryError> {
        Self::read_sys_file("capacity")?
            .trim_end()
            .parse::<u8>()
            .map_err(|x| BatteryError::ParseCapacity(x.to_string()))
    }

    fn read_sys_file(filename: &str) -> Result<String, BatteryError> {
        let filepath = Path::new(BASE_DIR).join("BAT0").join(filename);
        let mut file = fs::OpenOptions::new()
            .read(true)
            .open(&filepath)
            .map_err(|x| {
                BatteryError::ReadFile(filepath.to_string_lossy().to_string(), x.to_string())
            })?;
        let mut content = String::new();
        file.read_to_string(&mut content).map_err(|x| {
            BatteryError::ReadFile(filepath.to_string_lossy().to_string(), x.to_string())
        })?;
        Ok(content)
    }
}

pub fn get_battery(config: &Config) -> Result<(), BatteryError> {
    let battery = Battery::try_new(config).unwrap();
    println!("{}", serde_json::to_string(&battery).unwrap());
    Ok(())
}
