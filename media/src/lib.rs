use glue_traits::FunctionKey;
use souvlaki::{MediaControls, PlatformConfig};
use thiserror::Error;

pub struct Media {
    controls: MediaControls,
}

impl Media {
    fn new() -> Self {
        let config = PlatformConfig {
            display_name: "Glue Media Player",
            dbus_name: "glue_media_player",
            hwnd: None,
        };
        let controls = MediaControls::new(config).unwrap();
        Self { controls }
    }

    fn stop() -> Result<(), MediaError> {
        let mut media = Self::new();
        media
            .controls
            .set_playback(souvlaki::MediaPlayback::Stopped)
            .map_err(|err| MediaError::Stop(err.to_string()))
    }

    fn next() -> Result<(), MediaError> {
        let media = Self::new();
        todo!()
    }
}

#[derive(Error, Debug)]
pub enum MediaError {
    #[error("Unable to stop media: {:#?}", .0)]
    Stop(String),
}
