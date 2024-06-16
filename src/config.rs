use serde::{Deserialize, Serialize};
use toml;


#[derive(Deserialize, Serialize)]
pub struct Config {
    pub threshold: ThresholdConfig,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ThresholdConfig {
    pub brightness: f32,
}

pub fn load_config() -> Config {
    // Include the contents of config.toml at compile time
    // pwd:
    //let pwd = format!("{}/config.toml", std::env::current_dir().unwrap().to_str().unwrap());
    let config_str = include_str!("../config.toml");
    
    // Parse the config string into a Toml value or a specific config struct
    let config: Config = toml::from_str(config_str).expect("Failed to parse `config.toml` in root dir (where Cargo.toml is located)");

    config
}