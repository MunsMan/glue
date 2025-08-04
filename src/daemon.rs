use std::fs::File;
use std::ops::DerefMut;
use std::sync::Arc;
use std::time::Duration;

use glue_ipc::client::Client;
use glue_ipc::tokio::protocol::Protocol;
use log::{error, info};
use notify_rust::Notification;
use rand::distr::Alphanumeric;
use rand::{rng, Rng};
use tokio::sync::Mutex;
use tokio::time::interval;

use crate::autostart::auto_start;
use crate::coffee::{coffeinate, decoffeinate, CoffeeResponse};
use crate::commands::{self, Command};
use crate::configuration::Configuration;
use crate::error::{DaemonClientError, DaemonError};
use crate::eww::{self, eww_update};
use crate::monitor::Battery;
use crate::monitor::Monitor;
use crate::{hyprland, DaemonState, GLUE_PATH};

pub fn client(command: Command) -> Result<Vec<u8>, DaemonClientError> {
    let mut client = Client::new(GLUE_PATH).map_err(DaemonClientError::IPCError)?;
    client
        .send::<Command>(command)
        .map_err(DaemonClientError::IPCError)?;
    let message = client.read().map_err(DaemonClientError::IPCError)?;
    Ok(message)
}

#[tokio::main]
pub async fn daemon(
    config: &Configuration,
    _eww_config: Option<String>,
    no_autostart: bool,
) -> Result<(), DaemonError> {
    let daemon_id = daemon_id();
    setup_logging(config, &daemon_id)?;
    // eww::open(&eww::WindowName::Bar, eww_config.clone()).map_err(DaemonError::Command)?;
    if !no_autostart {
        auto_start(config).map_err(DaemonError::AutoStart)?;
    }

    let state = DaemonState::new(config.clone())?;

    tokio::try_join!(
        async {
            hyprland::listener(config.clone())
                .start_listener_async()
                .await
                .map_err(|err| DaemonError::Listener(err.to_string()))
        },
        server(GLUE_PATH, state, config.clone()),
        monitor(config)
    )?;
    Ok(())
}

pub fn daemon_id() -> String {
    rng()
        .sample_iter(&Alphanumeric)
        .take(12)
        .map(char::from)
        .collect()
}

fn setup_logging(config: &Configuration, daemon_id: &str) -> Result<(), DaemonError> {
    std::fs::create_dir_all("/tmp/glue")
        .map_err(|err| DaemonError::Setup("Creating tmp/glue", err.to_string()))?;
    simplelog::CombinedLogger::init(vec![
        simplelog::TermLogger::new(
            config.general.log_level,
            simplelog::Config::default(),
            simplelog::TerminalMode::Mixed,
            simplelog::ColorChoice::Auto,
        ),
        simplelog::WriteLogger::new(
            config.general.log_level,
            simplelog::Config::default(),
            File::create(format!("/tmp/glue/glue-{daemon_id}.log"))
                .map_err(|err| DaemonError::Setup("Creating Log File", err.to_string()))?,
        ),
    ])
    .map_err(|err| DaemonError::Setup("Init simplelog", err.to_string()))?;
    Ok(())
}

async fn monitor(config: &Configuration) -> Result<(), DaemonError> {
    let mut ticker = interval(Duration::from_secs(1));

    let mut services = [Battery::try_new(config).await.unwrap()];
    loop {
        for service in &mut services {
            let result = service.update().await;
            if let Err(err) = result {
                error!("Monitoring Error: {err}");
            }
        }
        ticker.tick().await;
    }
}

async fn server(
    socket: &str,
    state: DaemonState,
    config: Configuration,
) -> Result<(), DaemonError> {
    let state = Arc::new(Mutex::new(state));
    let server = glue_ipc::tokio::server::Server::new(socket)
        .await
        .map_err(DaemonError::SocketError)?;
    server
        .listen::<_, Command, _>(
            |command, state: Arc<Mutex<DaemonState>>, mut stream| {
                let config = config.clone();
                async move {
                    match command {
                        Command::Coffee(coffee) => match coffee {
                            commands::Coffee::Drink => {
                                info!("Drink Coffee");
                                let result = coffeinate(state.lock().await.deref_mut());
                                if let Err(err) = result {
                                    error!("{err}");
                                }
                            }
                            commands::Coffee::Relax => {
                                info!("I'm getting sleepy!");
                                let result = decoffeinate(state.lock().await.deref_mut());
                                if let Err(err) = result {
                                    error!("{err}");
                                }
                            }
                            commands::Coffee::Toggle => {
                                info!("Toggle Coffee State");
                                let result = {
                                    let mut state = state.lock().await;
                                    match &state.idle_inhibited {
                                        true => decoffeinate(&mut state),
                                        false => coffeinate(&mut state),
                                    }
                                };
                                if let Err(err) = result {
                                    error!("{err}");
                                }
                            }
                            commands::Coffee::Get => {
                                info!("Coffee Get Request");
                            }
                        },
                        Command::Notification(notification) => match notification {
                            commands::Notification::Test(text) => {
                                info!("Notification Test");
                                let result =
                                    Notification::new().summary("Glue Test").body(&text).show();
                                if let Err(error) = result {
                                    error!("Unable to send notification: {error:#?}");
                                }
                            }
                        },
                    };

                    match state.lock().await.wayland_idle.get() {
                        Ok(state) => {
                            let try_state_buffer = serde_json::to_vec(&state);
                            let Ok(state_buffer) = try_state_buffer else {
                                error!("Coffee Get: {try_state_buffer:#?}");
                                return;
                            };
                            let mut client = Protocol::new(&mut stream);
                            client.write_message(&state_buffer).await.unwrap();
                            if let Err(err) = eww_update(eww::EwwVariable::Coffee(
                                CoffeeResponse::new(&config, &state.into()),
                            )) {
                                error!("Unable to update EWW: {err:#?}");
                            };
                        }
                        Err(_err) => todo!(),
                    };
                }
            },
            state,
        )
        .await;
    Ok(())
}
