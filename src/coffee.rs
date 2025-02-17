use crate::{
    commands::{Coffee, Command},
    error::CoffeeError,
    wayland::WaylandIdle,
    DaemonState, GLUE_PATH,
};
use glue_ipc::client::Client;

/// Requires the daemon to hold the file descriper to block the system
/// from idling. Therefore IPC is required, which probably should be done
/// via a socket.

pub fn client(command: Coffee) -> Result<(), CoffeeError> {
    let mut client = Client::new(GLUE_PATH).map_err(CoffeeError::IPCError)?;
    client
        .send::<Command>(command.into())
        .map_err(CoffeeError::IPCError)?;
    let message = client.read().map_err(CoffeeError::IPCError)?;
    if !message.is_empty() {
        println!(
            "{}",
            serde_json::to_string(&serde_json::from_slice::<WaylandIdle>(&message).unwrap())
                .unwrap()
        );
    }
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
