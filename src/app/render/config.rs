use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub backends: String,
    pub force_fallback_adapter: Option<bool>,
}


