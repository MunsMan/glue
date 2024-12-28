use brightness::blocking::{brightness_devices, Brightness, BrightnessDevice};
use serde::Serialize;

use crate::{
    error::GlueError,
    eww::{eww_update, EwwVariable},
    key::FunctionKey,
    Change,
};

#[derive(Serialize, Clone)]
struct Device {
    name: String,
    brightness: u32,
}

pub(crate) struct BrightnessCtl {
    devices: Vec<(Device, BrightnessDevice)>,
}

#[derive(Serialize)]
pub(crate) struct BrightnessSettings {
    devices: Vec<Device>,
}

impl Into<BrightnessSettings> for &BrightnessCtl {
    fn into(self) -> BrightnessSettings {
        BrightnessSettings {
            devices: self.devices.iter().map(|x| x.0.clone()).collect(),
        }
    }
}

impl BrightnessCtl {
    fn new() -> Self {
        Self {
            devices: brightness_devices()
                .filter_map(|x| match x {
                    Ok(device) => {
                        let name = match device.device_name() {
                            Ok(name) => name,
                            Err(_) => return None,
                        };
                        let brightness = match device.get() {
                            Ok(brightness) => brightness,
                            Err(_) => return None,
                        };
                        Some((Device { name, brightness }, device))
                    }
                    Err(_) => None,
                })
                .collect(),
        }
    }

    fn update(&self) {
        eww_update(EwwVariable::Brightness(self.into())).unwrap();
    }

    fn change(&mut self, change: Change<u32>) -> Result<(), GlueError> {
        for (device, controller) in self.devices.iter() {
            let mut brightness = device.brightness;
            brightness = match change {
                Change::Add(update) => (brightness + update).min(100),
                Change::Sub(div) => match brightness.checked_sub(div) {
                    Some(result) => result,
                    None => 0,
                },
                Change::Absolute(value) => (value).min(100),
            };
            controller.set(brightness).map_err(GlueError::Brightness)?;
        }
        Ok(())
    }

    pub fn set(value: u32) -> Result<(), GlueError> {
        Self::new().change(Change::Absolute(value))?;
        Ok(())
    }
}

impl FunctionKey for BrightnessCtl {
    fn increase() -> Result<(), GlueError> {
        Self::new().change(Change::Add(5))?;
        Ok(())
    }

    fn decrease() -> Result<(), GlueError> {
        Self::new().change(Change::Sub(5))?;
        Ok(())
    }
}

pub fn get_brightness() -> Result<(), GlueError> {
    let brightness_ctl = BrightnessCtl::new();
    brightness_ctl
        .devices
        .iter()
        .for_each(|(device, _)| println!("{} - {}%", device.name, device.brightness));
    Ok(())
}
