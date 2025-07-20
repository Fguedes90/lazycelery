use lazycelery::config::{BrokerConfig, Config, UiConfig};
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_default_config() {
    let config = Config::default();

    assert_eq!(config.broker.url, "redis://localhost:6379/0");
    assert_eq!(config.broker.timeout, 30);
    assert_eq!(config.broker.retry_attempts, 3);
    assert_eq!(config.ui.refresh_interval, 1000);
    assert_eq!(config.ui.theme, "dark");
}

#[test]
fn test_config_from_file() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("test_config.toml");

    let config_content = r#"
[broker]
url = "redis://192.168.1.100:6379/1"
timeout = 60
retry_attempts = 5

[ui]
refresh_interval = 2000
theme = "light"
"#;

    fs::write(&config_path, config_content).unwrap();

    let config = Config::from_file(config_path).unwrap();

    assert_eq!(config.broker.url, "redis://192.168.1.100:6379/1");
    assert_eq!(config.broker.timeout, 60);
    assert_eq!(config.broker.retry_attempts, 5);
    assert_eq!(config.ui.refresh_interval, 2000);
    assert_eq!(config.ui.theme, "light");
}

#[test]
fn test_partial_config_from_file() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("partial_config.toml");

    let config_content = r#"
[broker]
url = "redis://custom:6379/0"

[ui]
refresh_interval = 500
"#;

    fs::write(&config_path, config_content).unwrap();

    // This should fail because required fields are missing
    let result = Config::from_file(config_path);
    assert!(result.is_err());
}

#[test]
fn test_invalid_config_file() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("invalid_config.toml");

    let config_content = "invalid toml content {{";

    fs::write(&config_path, config_content).unwrap();

    let result = Config::from_file(config_path);
    assert!(result.is_err());
}

#[test]
fn test_nonexistent_config_file() {
    let config_path = PathBuf::from("/nonexistent/path/config.toml");
    let result = Config::from_file(config_path);
    assert!(result.is_err());
}

#[test]
fn test_config_serialization() {
    let config = Config {
        broker: BrokerConfig {
            url: "amqp://guest:guest@localhost:5672//".to_string(),
            timeout: 45,
            retry_attempts: 2,
        },
        ui: UiConfig {
            refresh_interval: 3000,
            theme: "custom".to_string(),
        },
    };

    let toml_str = toml::to_string(&config).unwrap();
    let deserialized: Config = toml::from_str(&toml_str).unwrap();

    assert_eq!(config.broker.url, deserialized.broker.url);
    assert_eq!(config.broker.timeout, deserialized.broker.timeout);
    assert_eq!(config.ui.refresh_interval, deserialized.ui.refresh_interval);
    assert_eq!(config.ui.theme, deserialized.ui.theme);
}
