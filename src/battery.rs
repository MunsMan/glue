use std::fmt::Display;
use std::fs;
use std::io::Read;
use std::path::Path;

use serde::Serialize;

use crate::configuration::Configuration;
use crate::error::BatteryError;

const BASE_DIR: &str = "/sys/class/power_supply";

#[derive(Serialize)]
struct Battery {
    state: BatteryState,
    capacity: u8,
    icon: char,
}

impl Battery {
    fn try_new(config: &Configuration) -> Result<Self, BatteryError> {
        let state = Self::read_state(config.battery_path.as_deref())?;
        let capacity = Self::read_capacity(config.battery_path.as_deref())?;
        let icon = Self::icon(&state, capacity, config);
        Ok(Self {
            state,
            capacity,
            icon,
        })
    }

    fn icon(state: &BatteryState, capacity: u8, config: &Configuration) -> char {
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

    fn read_state(battery_path: Option<&str>) -> Result<BatteryState, BatteryError> {
        Self::read_sys_file("status", battery_path)?
            .trim_end()
            .try_into()
    }

    fn read_capacity(battery_path: Option<&str>) -> Result<u8, BatteryError> {
        Self::read_sys_file("capacity", battery_path)?
            .trim_end()
            .parse::<u8>()
            .map_err(|x| BatteryError::ParseCapacity(x.to_string()))
    }

    fn read_sys_file(filename: &str, battery_path: Option<&str>) -> Result<String, BatteryError> {
        let base_path = battery_path.unwrap_or(BASE_DIR);
        let filepath = Path::new(base_path).join("BAT0").join(filename);
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

pub fn get_battery(config: &Configuration) -> Result<String, BatteryError> {
    let battery = Battery::try_new(config).unwrap();
    Ok(serde_json::to_string(&battery).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configuration::{Battery as BatteryConfiguration, Configuration};
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    fn setup_test_environment() -> (TempDir, Configuration) {
        let temp_dir = TempDir::new().unwrap();
        let bat_dir = temp_dir.path().join("BAT0");
        std::fs::create_dir_all(&bat_dir).unwrap();

        // Create test battery files
        let mut status_file = File::create(bat_dir.join("status")).unwrap();
        writeln!(status_file, "Charging").unwrap();

        let mut capacity_file = File::create(bat_dir.join("capacity")).unwrap();
        writeln!(capacity_file, "75").unwrap();

        let config = Configuration {
            battery_path: Some(temp_dir.path().to_str().unwrap().to_string()),
            battery: BatteryConfiguration {
                charging: '⚡',
                empty: '💀',
                full: '🔋',
                charging_states: vec!['▁', '▂', '▃', '▄', '▅'],
            },
            ..Default::default()
        };

        (temp_dir, config)
    }

    #[test]
    fn test_get_battery() {
        let (_temp_dir, config) = setup_test_environment();
        // let mut output = TestLogger::new();

        let result = get_battery(&config);
        // Redirect stdout to our test logger
        // let result = with_stdout(&mut output, || get_battery(&config));

        assert!(result.is_ok());
        let output = result.unwrap();

        // Parse the JSON output
        let output_json: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(output_json["state"], "Charging");
        assert_eq!(output_json["capacity"], 75);
        assert_eq!(output_json["icon"], "⚡");
    }
}
