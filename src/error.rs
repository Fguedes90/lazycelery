use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum BrokerError {
    #[error("Connection failed: {0}")]
    ConnectionError(String),

    #[error("Authentication failed")]
    AuthError,

    #[error("Broker operation failed: {0}")]
    OperationError(String),

    #[error("Invalid broker URL: {0}")]
    InvalidUrl(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Timeout occurred")]
    Timeout,

    #[error("Not implemented")]
    NotImplemented,
}

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum AppError {
    #[error("Broker error: {0}")]
    Broker(#[from] BrokerError),

    #[error("UI error: {0}")]
    Ui(String),

    #[error("Configuration error: {0}")]
    Config(String),
}
