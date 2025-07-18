use std::fmt::Display;
use std::fs;
use std::io::Read;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::configuration::Configuration;
use crate::error::BatteryError;

#[derive(Serialize)]
struct Battery {
    state: BatteryStatus,
    capacity: u8,
    icon: char,
}

impl Battery {
    fn try_new(config: &Configuration) -> Result<Self, BatteryError> {
        let state = Self::read_state(&config.battery.path)?;
        let capacity = Self::read_capacity(&config.battery.path)?;
        let icon = Self::icon(&state, capacity, config);
        Ok(Self {
            state,
            capacity,
            icon,
        })
    }

    fn icon(state: &BatteryStatus, capacity: u8, config: &Configuration) -> char {
        match state {
            BatteryStatus::NotCharging => config.battery.full,
            BatteryStatus::Full => config.battery.full,
            BatteryStatus::Discharging => {
                let icons = &config.battery.charging_states;
                let index = capacity / (100 / icons.len() as u8);
                *icons.get(index as usize).unwrap_or(icons.last().unwrap())
            }
            BatteryStatus::Charging => config.battery.charging,
            BatteryStatus::Empty => config.battery.empty,
        }
    }

    fn read_state(battery_path: &str) -> Result<BatteryStatus, BatteryError> {
        Self::read_sys_file("status", battery_path)?
            .trim_end()
            .try_into()
    }

    fn read_capacity(battery_path: &str) -> Result<u8, BatteryError> {
        Self::read_sys_file("capacity", battery_path)?
            .trim_end()
            .parse::<u8>()
            .map_err(|x| BatteryError::ParseCapacity(x.to_string()))
    }

    fn read_sys_file(filename: &str, battery_path: &str) -> Result<String, BatteryError> {
        let filepath = Path::new(battery_path).join(filename);
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub(crate) enum BatteryStatus {
    Charging,
    Discharging,
    Empty,
    Full,
    NotCharging,
}

impl TryFrom<&str> for BatteryStatus {
    type Error = BatteryError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Charging" => Ok(BatteryStatus::Charging),
            "Discharging" => Ok(BatteryStatus::Discharging),
            "Empty" => Ok(BatteryStatus::Empty),
            "Full" => Ok(BatteryStatus::Full),
            "Not charging" => Ok(BatteryStatus::NotCharging),
            x => Err(BatteryError::UnknownState(x.to_string())),
        }
    }
}

impl From<&BatteryStatus> for String {
    fn from(val: &BatteryStatus) -> Self {
        match val {
            BatteryStatus::Charging => "Charging".to_string(),
            BatteryStatus::Discharging => "Discharging".to_string(),
            BatteryStatus::Empty => "Empty".to_string(),
            BatteryStatus::Full => "Full".to_string(),
            BatteryStatus::NotCharging => "Not charging".to_string(),
        }
    }
}

impl Display for BatteryStatus {
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
            battery: BatteryConfiguration {
                charging: '⚡',
                empty: '💀',
                full: '🔋',
                charging_states: vec!['▁', '▂', '▃', '▄', '▅'],
                path: bat_dir.to_string_lossy().to_string(),
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
