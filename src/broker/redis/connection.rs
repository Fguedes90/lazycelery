use crate::error::BrokerError;
use redis::aio::MultiplexedConnection;
use redis::Client;
use tracing::{debug, error, info};

/// Legacy connection wrapper - deprecated in favor of BrokerFacade
/// This is kept for backward compatibility but should not be used directly
#[deprecated(note = "Use BrokerFacade instead for better connection management")]
pub struct RedisConnection {
    #[deprecated(note = "Use BrokerFacade instead for better connection management")]
    #[allow(dead_code)]
    client: Client,
    #[deprecated(note = "Use BrokerFacade instead for better connection management")]
    #[allow(dead_code)]
    connection: MultiplexedConnection,
}

#[allow(deprecated)]
impl RedisConnection {
    #[allow(dead_code)]
    pub async fn new(url: &str) -> Result<Self, BrokerError> {
        debug!("Creating legacy RedisConnection - consider using BrokerFacade instead");

        let client = Client::open(url).map_err(|e| {
            error!("Failed to create Redis client: {}", e);
            BrokerError::InvalidUrl(format!("Invalid Redis URL: {e}"))
        })?;

        let connection = client
            .get_multiplexed_tokio_connection()
            .await
            .map_err(|e| {
                error!("Failed to establish Redis connection: {}", e);
                BrokerError::ConnectionError(format!("Connection failed: {e}"))
            })?;

        info!("Legacy RedisConnection established successfully");

        Ok(Self { client, connection })
    }

    /// Internal method for backward compatibility - do not use in new code
    /// Use BrokerFacade instead which provides proper connection pooling
    #[doc(hidden)]
    #[allow(dead_code)]
    pub(crate) fn get_connection(&self) -> MultiplexedConnection {
        debug!("Using legacy get_connection() - consider migrating to BrokerFacade");
        self.connection.clone()
    }
}
