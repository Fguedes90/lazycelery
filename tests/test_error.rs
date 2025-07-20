use lazycelery::error::{BrokerError, AppError};

#[test]
fn test_broker_error_display() {
    let err = BrokerError::ConnectionError("Failed to connect".to_string());
    assert_eq!(err.to_string(), "Connection failed: Failed to connect");
    
    let err = BrokerError::AuthError;
    assert_eq!(err.to_string(), "Authentication failed");
    
    let err = BrokerError::OperationError("Operation failed".to_string());
    assert_eq!(err.to_string(), "Broker operation failed: Operation failed");
    
    let err = BrokerError::InvalidUrl("bad url".to_string());
    assert_eq!(err.to_string(), "Invalid broker URL: bad url");
    
    let err = BrokerError::Timeout;
    assert_eq!(err.to_string(), "Timeout occurred");
    
    let err = BrokerError::NotImplemented;
    assert_eq!(err.to_string(), "Not implemented");
}

#[test]
fn test_app_error_from_broker_error() {
    let broker_err = BrokerError::ConnectionError("test".to_string());
    let app_err: AppError = broker_err.into();
    
    match app_err {
        AppError::Broker(_) => {},
        _ => panic!("Expected AppError::Broker variant"),
    }
}

#[test]
fn test_app_error_display() {
    let broker_err = BrokerError::AuthError;
    let app_err = AppError::Broker(broker_err);
    assert_eq!(app_err.to_string(), "Broker error: Authentication failed");
    
    let app_err = AppError::Ui("UI crashed".to_string());
    assert_eq!(app_err.to_string(), "UI error: UI crashed");
    
    let app_err = AppError::Config("Invalid config".to_string());
    assert_eq!(app_err.to_string(), "Configuration error: Invalid config");
}
