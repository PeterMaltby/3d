use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::path::PathBuf;

use crate::app;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub renderer: app::render::config::Config,
    pub rust_log: Option<String>,
    pub fullscreen: Option<bool>,
}


impl AppConfig {
    pub fn new(config_path: Option<String>) -> Result<Self, ConfigError> {
        let config_path = match config_path {
            Some(config_path) => PathBuf::from(config_path),
            _ => PathBuf::from("config"),
        };

        return Config::builder()
            .add_source(File::from(config_path))
            .add_source(Environment::with_prefix("APP"))
            .build()?
            .try_deserialize();
    }
}
