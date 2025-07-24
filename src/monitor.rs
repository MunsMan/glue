use async_trait::async_trait;
use std::{ops::Deref, path::Path, sync::Arc};

use crate::{
    battery::BatteryStatus,
    configuration::{BatteryEvent, Configuration},
    error::{BatteryError, DaemonError, GlueError},
    eww::eww_update,
};
use log::{error, info};
use notify_rust::Notification;
use serde::Serialize;
use tokio::{fs::OpenOptions, io::AsyncReadExt};

#[async_trait]
pub(crate) trait Monitor {
    async fn update(&mut self) -> Result<(), GlueError>;
    async fn event(&self);
}

pub(crate) enum Event {
    Battery(BatteryEvent),
}

pub(crate) struct Battery {
    config: Arc<Configuration>,
    path: String,
    status: BatteryStatus,
    capacity: u8,
    events: Vec<Event>,
}

#[derive(Serialize)]
pub(crate) struct BatteryState {
    status: BatteryStatus,
    capacity: u8,
    icon: char,
}

impl From<&Battery> for BatteryState {
    fn from(value: &Battery) -> Self {
        Self {
            status: value.status,
            capacity: value.capacity,
            icon: value.icon(),
        }
    }
}

#[async_trait]
impl Monitor for Battery {
    async fn update(&mut self) -> Result<(), GlueError> {
        let (capacity, status) = Self::read_state(&self.path)
            .await
            .map_err(GlueError::Battery)?;
        if (self.status != status) || (self.capacity != capacity) {
            info!(
                "capacity: {} - old: {}, status: {} - old: {}",
                capacity, self.capacity, status, self.status
            );
            self.capacity = capacity;
            self.status = status;
            self.event().await;
            return eww_update(crate::eww::EwwVariable::Battery(self.deref().into()))
                .map_err(GlueError::Command);
        }
        Ok(())
    }

    async fn event(&self) {
        for Event::Battery(event) in &self.events {
            if event.state == self.status && event.charge == self.capacity {
                if let Some(text) = &event.notify {
                    let _ = Notification::new()
                        .summary("Battery")
                        .body(text)
                        .timeout(0)
                        .show();
                }
            }
        }
    }
}

pub(crate) async fn monitor(services: &mut Vec<Box<dyn Monitor>>) -> Result<(), DaemonError> {
    for service in services {
        let result = service.update().await;
        if let Err(err) = result {
            error!("Monitoring Error: {}", err);
        }
    }
    Ok(())
}

type BatteryCapacity = u8;

impl Battery {
    pub(crate) async fn try_new(config: Arc<Configuration>) -> Result<Self, GlueError> {
        let mut battery = Battery::new(config).map_err(GlueError::Battery)?;
        battery.update().await?;
        Ok(battery)
    }

    fn new(config: Arc<Configuration>) -> Result<Self, BatteryError> {
        let mut events = Vec::new();
        if let Some(all_events) = &config.event {
            all_events
                .battery
                .iter()
                .for_each(|event| events.push(Event::Battery(event.clone())));
        }
        Ok(Self {
            path: config.battery.path.clone(),
            status: BatteryStatus::Empty,
            capacity: 0,
            config,
            events,
        })
    }

    async fn read_state(path: &str) -> Result<(BatteryCapacity, BatteryStatus), BatteryError> {
        let (raw_capacity, raw_status) = tokio::try_join!(
            Self::read_sys(path, "capacity"),
            Self::read_sys(path, "status")
        )?;
        let capacity: BatteryCapacity = raw_capacity
            .trim_end()
            .parse::<u8>()
            .map_err(|x| BatteryError::ParseCapacity(x.to_string()))?;
        let status: BatteryStatus = raw_status.trim_end().try_into()?;
        Ok((capacity, status))
    }

    async fn read_sys(path: &str, filename: &str) -> Result<String, BatteryError> {
        let path = Path::new(path).join(filename);
        let mut file = OpenOptions::new()
            .read(true)
            .open(&path)
            .await
            .map_err(|x| {
                BatteryError::ReadFile(path.to_string_lossy().to_string(), x.to_string())
            })?;
        let mut content = String::new();
        file.read_to_string(&mut content).await.map_err(|x| {
            BatteryError::ReadFile(path.to_string_lossy().to_string(), x.to_string())
        })?;
        Ok(content)
    }

    fn icon(&self) -> char {
        match self.status {
            BatteryStatus::NotCharging => self.config.battery.full,
            BatteryStatus::Full => self.config.battery.full,
            BatteryStatus::Discharging => {
                let icons = &self.config.battery.charging_states;
                let index = self.capacity / (100 / icons.len() as u8);
                *icons.get(index as usize).unwrap_or(icons.last().unwrap())
            }
            BatteryStatus::Charging => self.config.battery.charging,
            BatteryStatus::Empty => self.config.battery.empty,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::*;
    use crate::configuration::{Battery as BatteryConfiguration, Configuration, Events};
    use std::io::Write;
    use tempfile::TempDir;

    fn setup_test_environment() -> (Configuration, File, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let bat_dir = temp_dir.path().join("BAT0");
        std::fs::create_dir_all(&bat_dir).unwrap();

        // Create test battery files
        let mut status_file = File::create(bat_dir.join("status")).unwrap();
        writeln!(status_file, "Discharging").unwrap();

        let mut capacity_file = File::create(bat_dir.join("capacity")).unwrap();
        writeln!(capacity_file, "21").unwrap();

        let config = Configuration {
            battery: BatteryConfiguration {
                path: bat_dir.to_string_lossy().to_string(),
                ..Default::default()
            },
            event: Some(Events {
                battery: vec![BatteryEvent {
                    charge: 20,
                    state: BatteryStatus::Discharging,
                    notify: None,
                    shell: None,
                    hooks: None,
                }],
            }),
            ..Default::default()
        };

        (config, capacity_file, temp_dir)
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_battery_notification() {
        let (config, mut capacity_file, _temp_dir) = setup_test_environment();
        let battery = Battery::try_new(config.into()).await.unwrap();
        let mut services: Vec<Box<dyn Monitor>> = vec![Box::new(battery)];
        let result = monitor(&mut services).await;
        assert!(result.is_ok());
        write!(capacity_file, "20").unwrap();
        let result = monitor(&mut services).await;
        assert!(result.is_ok());
    }
}
