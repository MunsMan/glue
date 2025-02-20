use std::fs::File;

use glue_ipc::tokio::protocol::Protocol;
use log::{error, info};
use rand::distr::Alphanumeric;
use rand::{rng, Rng};

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
    auto_start(config).map_err(|x| DaemonError::AutoStart(x))?;

    let state = DaemonState::new(config.clone())?;

    tokio::try_join!(
        async {
            hyprland::listener(config.clone())
                .start_listener_async()
                .await
                .map_err(|err| DaemonError::Listener(err.to_string()))
        },
        server(&GLUE_PATH, state, config.clone())
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
    let server = glue_ipc::tokio::server::Server::new(socket)
        .await
        .map_err(DaemonError::SocketError)?;
    server
        .listen::<_, Command, _, _>(
            |command, mut state, mut stream| {
                let config = config.clone();
                async move {
                    match command {
                        Command::Coffee(coffee) => match coffee {
                            commands::Coffee::Drink => {
                                info!("Drink Coffee");
                                let result = coffeinate(&mut state);
                                if let Err(err) = result {
                                    error!("{}", err);
                                }
                            }
                            commands::Coffee::Relax => {
                                info!("I'm getting sleepy!");
                                let result = decoffeinate(&mut state);
                                if let Err(err) = result {
                                    error!("{}", err);
                                }
                            }
                            commands::Coffee::Toggle => {
                                info!("Toggle Coffee State");
                                let result = state.wayland_idle.toggle();
                                if let Err(err) = result {
                                    error!("{}", err);
                                }
                            }
                            commands::Coffee::Get => {
                                info!("Coffee Get Request");
                            }
                        },
                    };
                    match state.wayland_idle.get() {
                        Ok(state) => {
                            let result = serde_json::to_vec(&state);
                            let Ok(buffer) = result else {
                                error!("Coffee Get: {:#?}", result);
                                return;
                            };
                            let mut protocol = Protocol::new(&mut stream);
                            protocol.write_message(&buffer).await.unwrap();
                            if let Err(err) = eww_update(eww::EwwVariable::Coffee(
                                CoffeeResponse::new(&config, &state),
                            )) {
                                error!("Unable to update EWW: {:#?}", err);
                            };
                        }
                        Err(err) => error!("{}", err),
                    };
                }
            },
            state,
        )
        .await;
    Ok(())
}
