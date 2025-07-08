use std::sync::{Arc, Mutex};

use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use wayland_client::protocol::wl_display::WlDisplay;
use wayland_client::protocol::wl_registry::{self, WlRegistry};
use wayland_client::protocol::{wl_compositor::WlCompositor, wl_surface::WlSurface};
use wayland_client::{
    ConnectError as WLConnectionError, Dispatch, DispatchError, Proxy, QueueHandle,
};
use wayland_client::{Connection, EventQueue};
use wayland_protocols::wp::idle_inhibit::zv1::client::{
    zwp_idle_inhibit_manager_v1::ZwpIdleInhibitManagerV1, zwp_idle_inhibitor_v1::ZwpIdleInhibitorV1,
};

#[derive(Error, Debug)]
pub enum WaylandClientError {
    #[error("Unable to establish a Connection to the Wayland Server: {}", .0)]
    ConnectionError(WLConnectionError),
    #[error("Roundtrip faild: {}", .0)]
    RoundTripError(DispatchError),
}

#[derive(Default, Clone, Debug)]
struct WaylandAppData {
    compositor: Option<(WlCompositor, u32)>,
    surface: Option<WlSurface>,
    idle_manager: Option<(ZwpIdleInhibitManagerV1, u32)>,
    idle_inhibitor: Option<ZwpIdleInhibitorV1>,
}

impl Dispatch<ZwpIdleInhibitManagerV1, ()> for WaylandAppData {
    fn event(
        _state: &mut Self,
        _proxy: &ZwpIdleInhibitManagerV1,
        _event: <ZwpIdleInhibitManagerV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpIdleInhibitorV1, ()> for WaylandAppData {
    fn event(
        _state: &mut Self,
        _proxy: &ZwpIdleInhibitorV1,
        _event: <ZwpIdleInhibitorV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlCompositor, ()> for WaylandAppData {
    fn event(
        _state: &mut Self,
        _proxy: &WlCompositor,
        _event: <WlCompositor as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlSurface, ()> for WaylandAppData {
    fn event(
        _state: &mut Self,
        _proxy: &WlSurface,
        _event: <WlSurface as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlRegistry, ()> for WaylandAppData {
    fn event(
        state: &mut Self,
        proxy: &WlRegistry,
        event: <WlRegistry as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        match event {
            wl_registry::Event::Global {
                name,
                interface,
                version,
            } => {
                if interface == WlCompositor::interface().name && state.compositor.is_none() {
                    debug!(target: "WaylandIdleInhibitor::WlRegister::Event::Global", "Adding Compositor");
                    let compositor: WlCompositor = proxy.bind(name, version, qhandle, ());
                    state.surface = Some(compositor.create_surface(qhandle, ()));
                    state.compositor = Some((compositor, name))
                } else if interface == ZwpIdleInhibitManagerV1::interface().name
                    && state.idle_manager.is_none()
                {
                    debug!(target: "WaylandIdleInhibitor::WlRegister::Event::Global", "Adding IdleInhibitManager");
                    state.idle_manager = Some((proxy.bind(name, version, qhandle, ()), name))
                }
            }
            wl_registry::Event::GlobalRemove { name } => {
                if let Some((_, compositer_name)) = &state.compositor {
                    if name == *compositer_name {
                        warn!(target: "WaylandIdleInhibitor::GlobalRemove", "Compositor was removed!");
                        state.compositor = None;
                        state.surface = None;
                    }
                }
                if let Some((_, idle_manager_name)) = &state.idle_manager {
                    if name == *idle_manager_name {
                        warn!(target: "WaylandIdleInhibitor::GlobalRemove", "IdleInhibitManager was removed!");
                        state.idle_manager = None;
                    }
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone)]
pub struct WaylandClient {
    _connection: Connection,
    event_queue: Arc<Mutex<EventQueue<WaylandAppData>>>,
    qhandle: QueueHandle<WaylandAppData>,
    app_data: WaylandAppData,
    _display: WlDisplay,
    _registry: WlRegistry,
}

impl WaylandClient {
    pub fn new() -> Result<Self, WaylandClientError> {
        let connection =
            Connection::connect_to_env().map_err(WaylandClientError::ConnectionError)?;
        let display = connection.display();
        let event_queue = Arc::new(Mutex::new(connection.new_event_queue()));
        let qhandle = event_queue.lock().unwrap().handle();
        let mut app_data = WaylandAppData::default();
        let registry = display.get_registry(&qhandle, ());
        event_queue
            .lock()
            .unwrap()
            .roundtrip(&mut app_data)
            .map_err(WaylandClientError::RoundTripError)?;
        Ok(WaylandClient {
            app_data,
            qhandle,
            _connection: connection,
            _registry: registry,
            _display: display,
            event_queue,
        })
    }

    pub fn inhibit(&mut self) -> Result<(), WaylandClientError> {
        // let Some(surface) = self.client.app_data.surface;
        let data = self.app_data.clone();
        let Some((idle_manager, _)) = &data.idle_manager else {
            warn!(target: "WaylandIdleInhibitor::set_inhibit_idle", "Tried to change idle inhibitor status without loaded idle inhibitor manager!");
            return Ok(());
        };

        if data.idle_inhibitor.is_none() {
            let Some(surface) = &data.surface else {
                warn!(target: "WaylandIdleInhibitor::set_inhibit_idle", "Tried to change idle inhibitor status without loaded WlSurface!");
                return Ok(());
            };
            self.app_data.idle_inhibitor =
                Some(idle_manager.create_inhibitor(surface, &self.qhandle, ()));
            self.event_queue
                .lock()
                .unwrap()
                .roundtrip(&mut self.app_data)
                .map_err(WaylandClientError::RoundTripError)?;
            info!(target: "WaylandIdleInhibitor::set_inhibit_idle", "Idle Inhibitor was ENABLED");
        }
        Ok(())
    }

    pub fn get(&mut self) -> Result<WaylandIdle, WaylandClientError> {
        let inhibited = self.app_data.idle_inhibitor.is_some();
        Ok(WaylandIdle { inhibited })
    }

    pub fn release(&mut self) -> Result<(), WaylandClientError> {
        if let Some(indle_inhibitor) = &self.app_data.idle_inhibitor {
            indle_inhibitor.destroy();
            self.app_data.idle_inhibitor = None;
            self.event_queue
                .lock()
                .unwrap()
                .roundtrip(&mut self.app_data)
                .map_err(WaylandClientError::RoundTripError)?;
            info!(target: "WaylandIdleInhibitor::set_inhibit_idle", "Idle Inhibitor was DISABLED");
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WaylandIdle {
    pub inhibited: bool,
}
