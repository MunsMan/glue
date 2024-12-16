use crate::error::SystemError;
use std::time::Duration;

use serde::Serialize;
use systemstat::{Platform, System};

#[derive(Serialize)]
struct MemoryState {
    total: u64,
    free: u64,
    used: f32,
}

impl MemoryState {
    fn new<T: Platform>(system: &T) -> Result<Self, SystemError> {
        let memory = system.memory().map_err(SystemError::MemoryError)?;
        let total = memory.total.as_u64() / 1024;
        let free = memory.free.as_u64() / 1024;
        Ok(Self {
            total,
            free,
            used: (total - free) as f32 / total as f32,
        })
    }
}

#[derive(Serialize)]
struct CpuState {
    user: f32,
    system: f32,
    idle: f32,
}

impl CpuState {
    fn new<T: Platform>(system: &T) -> Result<Self, SystemError> {
        let cpu = system.cpu_load_aggregate().map_err(SystemError::CpuError)?;
        std::thread::sleep(Duration::from_secs(1));
        let cpu = cpu.done().map_err(SystemError::CpuError)?;
        Ok(Self {
            user: cpu.user,
            system: cpu.system,
            idle: cpu.idle,
        })
    }
}

#[derive(Serialize)]
struct SystemState {
    memory: MemoryState,
    cpu: CpuState,
}

impl SystemState {
    fn new() -> Result<Self, SystemError> {
        let system = System::new();
        Ok(Self {
            memory: MemoryState::new(&system)?,
            cpu: CpuState::new(&system)?,
        })
    }
}

pub fn system() -> Result<(), SystemError> {
    print!(
        "{}",
        serde_json::to_string(&(SystemState::new())?)
            .map_err(|err| SystemError::SerializenError(err.to_string()))?
    );
    Ok(())
}
