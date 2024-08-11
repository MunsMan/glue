use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct Config {
    pub battery: Battery,
}

#[derive(Deserialize)]
pub struct Battery {
    pub charging_states: Vec<char>,
    pub full: char,
}

impl Config {
    pub fn load() -> Config {
        Config::default()
    }
}

impl Default for Battery {
    fn default() -> Self {
        Self {
            charging_states: vec!['', '', '', '', ''],
            full: '󱐥',
        }
    }
}
