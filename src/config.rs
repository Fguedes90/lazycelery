use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub broker: BrokerConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerConfig {
    pub url: String,
    pub timeout: u32,
    pub retry_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub refresh_interval: u64, // milliseconds
    pub theme: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            broker: BrokerConfig {
                url: "redis://localhost:6379/0".to_string(),
                timeout: 30,
                retry_attempts: 3,
            },
            ui: UiConfig {
                refresh_interval: 1000,
                theme: "dark".to_string(),
            },
        }
    }
}

impl Config {
    pub fn from_file(path: PathBuf) -> Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }
}
