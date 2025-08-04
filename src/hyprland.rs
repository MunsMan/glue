use hyprland::event_listener::EventListener;
use log::info;
use std::{sync::Arc, time::Duration};

use crate::{configuration::Configuration, wake_up, workspace::eww_workspace_update};

pub fn listener(config: Arc<Configuration>) -> EventListener {
    let mut listener = EventListener::new();
    let default_spaces = config.hyprland.default_spaces;
    listener.add_workspace_changed_handler(move |_| {
        info!("Workspace Switch");
        eww_workspace_update(default_spaces).expect("Unable to update workspace!")
    });
    let eww_config_monitor_add = config.general.eww_config.clone();
    listener.add_monitor_added_handler(move |data| {
        info!("Monitor {} is added (id: {})", data.name, data.id);
        std::thread::sleep(Duration::from_secs(5));
        wake_up(eww_config_monitor_add.clone()).expect("Unable to wake up glue!");
    });
    let eww_config_monitor_remove = config.general.eww_config.clone();
    listener.add_monitor_removed_handler(move |data| {
        info!("Monitor {} is removed", data);
        wake_up(eww_config_monitor_remove.clone()).expect("Unable to wake up glue!");
    });
    listener
}
