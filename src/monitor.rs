use std::path::Path;

use crate::{
    battery::BatteryStatus,
    configuration::Configuration,
    error::{BatteryError, GlueError},
    eww::eww_update,
};
use serde::Serialize;
use tokio::{fs::OpenOptions, io::AsyncReadExt};

pub(crate) trait Monitor<'a>: std::marker::Sized {
    async fn try_new(config: &'a Configuration) -> Result<Self, GlueError>;
    async fn update(&self) -> Result<(), GlueError>;
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
        Battery::try_new(config).await.map_err(GlueError::Battery)
    }

    async fn update(&self) -> Result<(), GlueError> {
        let (capacity, status) = Self::read_state(&self.path)
            .await
            .map_err(GlueError::Battery)?;
        if self.status != status || self.capacity != capacity {
            return eww_update(crate::eww::EwwVariable::Battery(self.into()))
                .map_err(GlueError::Command);
        }
        Ok(())
    }
}

type BatteryCapacity = u8;

impl<'a> Battery<'a> {
    async fn try_new(config: &'a Configuration) -> Result<Self, BatteryError> {
        let (capacity, status) = Self::read_state(&config.battery.path).await?;
        Ok(Self {
            path: config.battery.path.clone(),
            status,
            capacity,
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
