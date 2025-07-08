use crate::{
    commands::Coffee, configuration::Configuration, daemon, error::CoffeeError,
    utils::CancelableTimer, wayland::WaylandIdle, DaemonState, IdleState,
};
use serde::Serialize;

/*
Requires the daemon to hold the file descriper to block the system
from idling. Therefore IPC is required, which probably should be done
via a socket.
*/
/// Response Object for all coffee commands
#[derive(Serialize)]
pub struct CoffeeResponse {
    inhibited: bool,
    icon: char,
}

impl CoffeeResponse {
    pub fn new(configuration: &Configuration, state: &IdleState) -> Self {
        Self {
            inhibited: state.inhibited,
            icon: if state.inhibited {
                configuration.coffee.coffee
            } else {
                configuration.coffee.relax
            },
        }
    }
}

/// IPC coffee client, which forwards commands to the daemon
pub fn client(command: Coffee, configuration: &Configuration) -> Result<(), CoffeeError> {
    let message = daemon::client(command.into()).map_err(CoffeeError::IPCError)?;
    if !message.is_empty() {
        let state = serde_json::from_slice::<WaylandIdle>(&message).unwrap();
        println!(
            "{}",
            serde_json::to_string(&CoffeeResponse::new(configuration, &state.into())).unwrap()
        );
    }
    Ok(())
}

pub fn coffeinate(state: &mut DaemonState) -> Result<(), CoffeeError> {
    state
        .wayland_idle
        .inhibit()
        .map_err(CoffeeError::WaylandError)?;
    if let Some(notify) = state.notification {
        let timer = CancelableTimer::new(notify);
        timer.start();
        state.idle_notify = Some(timer);
    }
    Ok(())
}

pub fn decoffeinate(state: &mut DaemonState) -> Result<(), CoffeeError> {
    state
        .wayland_idle
        .release()
        .map_err(CoffeeError::WaylandError)?;
    if let Some(notification) = &state.idle_notify {
        notification.cancel();
        state.idle_notify = None;
    }
    Ok(())
}
