use crate::{
    commands::{Coffee, Command},
    error::CoffeeError,
    DaemonState, GLUE_PATH,
};
use glue_ipc::client::Client;

/// Requires the daemon to hold the file descriper to block the system
/// from idling. Therefore IPC is required, which probably should be done
/// via a socket.

pub fn drink() -> Result<(), CoffeeError> {
    let mut client = Client::new(GLUE_PATH).map_err(CoffeeError::IPCError)?;
    client
        .send::<Command>(Coffee::Drink.into())
        .map_err(CoffeeError::IPCError)?;
    Ok(())
}

pub fn relax() -> Result<(), CoffeeError> {
    let mut client = Client::new(GLUE_PATH).map_err(CoffeeError::IPCError)?;
    client
        .send::<Command>(Coffee::Relex.into())
        .map_err(CoffeeError::IPCError)?;
    Ok(())
}

pub fn coffeinate(state: &mut DaemonState) -> Result<(), CoffeeError> {
    state
        .wayland_idle
        .inhibit()
        .map_err(CoffeeError::WaylandError)
}

pub fn decoffeinate(state: &mut DaemonState) -> Result<(), CoffeeError> {
    state
        .wayland_idle
        .release()
        .map_err(CoffeeError::WaylandError)
}
