use std::fs::File;
use std::ops::DerefMut;
use std::sync::Arc;

use glue_ipc::tokio::protocol::Protocol;
use log::{error, info};
use rand::distr::Alphanumeric;
use rand::{rng, Rng};
use tokio::sync::Mutex;

use crate::autostart::auto_start;
use crate::coffee::{coffeinate, decoffeinate, CoffeeResponse};
use crate::commands::{self, Command};
use crate::configuration::Configuration;
use crate::error::DaemonError;
use crate::eww::{self, eww_update};
use crate::{hyprland, DaemonState, GLUE_PATH};

#[tokio::main]
pub async fn daemon(config: &Configuration, eww_config: Option<String>) -> Result<(), DaemonError> {
    let daemon_id = daemon_id();
    setup_logging(config, &daemon_id)?;
    eww::open(&eww::WindowName::Bar, eww_config.clone()).map_err(DaemonError::Command)?;
    auto_start(config).map_err(DaemonError::AutoStart)?;

    let state = DaemonState::new(config.clone())?;

    tokio::try_join!(
        async {
            hyprland::listener(config.clone())
                .start_listener_async()
                .await
                .map_err(|err| DaemonError::Listener(err.to_string()))
        },
        server(GLUE_PATH, state, config.clone())
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
            File::create(format!("/tmp/glue/glue-{}.log", daemon_id))
                .map_err(|err| DaemonError::Setup("Creating Log File", err.to_string()))?,
        ),
    ])
    .map_err(|err| DaemonError::Setup("Init simplelog", err.to_string()))?;
    Ok(())
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
                                state.lock().await.idle_inhibited = true;
                                let result = coffeinate(state.lock().await.deref_mut());
                                if let Err(err) = result {
                                    error!("{}", err);
                                }
                            }
                            commands::Coffee::Relax => {
                                info!("I'm getting sleepy!");
                                state.lock().await.idle_inhibited = false;
                                let result = decoffeinate(state.lock().await.deref_mut());
                                if let Err(err) = result {
                                    error!("{}", err);
                                }
                            }
                            commands::Coffee::Toggle => {
                                info!("Toggle Coffee State");
                                let result = {
                                    let mut state = state.lock().await;
                                    state.idle_inhibited = !state.idle_inhibited;
                                    state.wayland_idle.toggle()
                                };
                                if let Err(err) = result {
                                    error!("{}", err);
                                }
                            }
                            commands::Coffee::Get => {
                                info!("Coffee Get Request");
                            }
                        },
                    };

                    match state.lock().await.wayland_idle.get() {
                        Ok(state) => {
                            let try_state_buffer = serde_json::to_vec(&state);
                            let Ok(state_buffer) = try_state_buffer else {
                                error!("Coffee Get: {:#?}", try_state_buffer);
                                return;
                            };
                            let mut client = Protocol::new(&mut stream);
                            client.write_message(&state_buffer).await.unwrap();
                            if let Err(err) = eww_update(eww::EwwVariable::Coffee(
                                CoffeeResponse::new(&config, &state.into()),
                            )) {
                                error!("Unable to update EWW: {:#?}", err);
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
