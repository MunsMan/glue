use std::{ops::Deref, path::Path};

use crate::{
    battery::BatteryStatus,
    configuration::{BatteryEvent, Configuration},
    error::{BatteryError, GlueError},
    eww::eww_update,
};
use log::info;
use notify_rust::Notification;
use serde::Serialize;
use tokio::{fs::OpenOptions, io::AsyncReadExt};

pub(crate) trait Monitor<'a>: std::marker::Sized {
    async fn try_new(config: &'a Configuration) -> Result<Self, GlueError>;
    async fn update(&mut self) -> Result<(), GlueError>;
    async fn event(&self, events: Vec<Event>);
}

pub(crate) enum Event {
    Battery(BatteryEvent),
}

pub(crate) struct Battery<'a> {
    config: &'a Configuration,
    path: String,
    status: BatteryStatus,
    capacity: u8,
}

#[derive(Serialize)]
pub(crate) struct BatteryState {
    status: BatteryStatus,
    capacity: u8,
    icon: char,
}

impl<'a> From<&Battery<'a>> for BatteryState {
    fn from(value: &Battery) -> Self {
        Self {
            status: value.status,
            capacity: value.capacity,
            icon: value.icon(),
        }
    }
}

impl<'a> Monitor<'a> for Battery<'a> {
    async fn try_new(config: &'a Configuration) -> Result<Self, GlueError> {
        let mut battery = Battery::try_new(config).await.map_err(GlueError::Battery)?;
        battery.update().await?;
        Ok(battery)
    }

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
            return eww_update(crate::eww::EwwVariable::Battery(self.deref().into()))
                .map_err(GlueError::Command);
        }
        Ok(())
    }

    async fn event(&self, events: Vec<Event>) {
        for Event::Battery(event) in events {
            if event.state == self.status && event.charge == self.capacity {
                if let Some(text) = event.notify {
                    let _ = Notification::new()
                        .summary("Battery")
                        .body(&text)
                        .timeout(0)
                        .show();
                }
            }
        }
    }
}

type BatteryCapacity = u8;

impl<'a> Battery<'a> {
    async fn try_new(config: &'a Configuration) -> Result<Self, BatteryError> {
        Ok(Self {
            path: config.battery.path.clone(),
            status: BatteryStatus::Empty,
            capacity: 0,
            config,
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
