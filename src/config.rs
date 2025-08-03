use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

    pub fn load_or_create_default() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
            .join("lazycelery");
        
        let config_path = config_dir.join("config.toml");
        
        if config_path.exists() {
            Self::from_file(config_path)
        } else {
            // Create default config
            let default_config = Self::default();
            
            // Try to create config directory and file
            if let Err(e) = std::fs::create_dir_all(&config_dir) {
                eprintln!("⚠️  Could not create config directory: {}", e);
            } else {
                let toml_string = toml::to_string_pretty(&default_config)?;
                if let Err(e) = std::fs::write(&config_path, toml_string) {
                    eprintln!("⚠️  Could not create config file: {}", e);
                } else {
                    eprintln!("✅ Created default config at: {}", config_path.display());
                }
            }
            
            Ok(default_config)
        }
    }
}
