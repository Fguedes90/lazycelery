use crate::error::BrokerError;
use redis::aio::MultiplexedConnection;
use redis::Client;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use tokio::time::sleep;

const DEFAULT_POOL_SIZE: usize = 10;
#[allow(dead_code)]
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);
const HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(30);
const MAX_RETRY_ATTEMPTS: u32 = 3;
const INITIAL_BACKOFF: Duration = Duration::from_millis(100);

#[derive(Debug)]
pub struct PooledConnection {
    pub connection: MultiplexedConnection,
    pub last_used: Instant,
    pub is_healthy: bool,
}

impl PooledConnection {
    fn new(connection: MultiplexedConnection) -> Self {
        Self {
            connection,
            last_used: Instant::now(),
            is_healthy: true,
        }
    }

    pub fn mark_used(&mut self) {
        self.last_used = Instant::now();
    }

    pub async fn health_check(&mut self) -> bool {
        // Simple ping to check if connection is alive
        let mut conn = self.connection.clone();
        match tokio::time::timeout(
            Duration::from_secs(1),
            redis::cmd("PING").query_async::<_, String>(&mut conn),
        )
        .await
        {
            Ok(Ok(_)) => {
                self.is_healthy = true;
                true
            }
            _ => {
                self.is_healthy = false;
                false
            }
        }
    }
}

pub struct ConnectionPool {
    client: Client,
    connections: Arc<Mutex<Vec<PooledConnection>>>,
    semaphore: Arc<Semaphore>,
    max_size: usize,
}

impl ConnectionPool {
    pub async fn new(url: &str, max_size: Option<usize>) -> Result<Self, BrokerError> {
        let client = Client::open(url)
            .map_err(|e| BrokerError::InvalidUrl(format!("Invalid Redis URL: {e}")))?;

        let max_size = max_size.unwrap_or(DEFAULT_POOL_SIZE);
        let connections = Arc::new(Mutex::new(Vec::with_capacity(max_size)));
        let semaphore = Arc::new(Semaphore::new(max_size));

        let pool = Self {
            client,
            connections,
            semaphore,
            max_size,
        };

        // Pre-populate pool with one connection to test connectivity
        pool.create_connection().await?;

        Ok(pool)
    }

    async fn create_connection(&self) -> Result<PooledConnection, BrokerError> {
        let connection = self
            .client
            .get_multiplexed_tokio_connection()
            .await
            .map_err(|e| {
                BrokerError::ConnectionError(format!("Failed to create connection: {e}"))
            })?;

        Ok(PooledConnection::new(connection))
    }

    pub async fn get_connection(&self) -> Result<MultiplexedConnection, BrokerError> {
        // Acquire semaphore permit first to limit concurrent connections
        let _permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| BrokerError::ConnectionError("Pool semaphore error".to_string()))?;

        // Try to get an existing healthy connection
        let mut connections = self.connections.lock().await;

        // Look for a healthy connection
        if let Some(index) = connections.iter().position(|conn| conn.is_healthy) {
            let mut pooled_conn = connections.remove(index);
            pooled_conn.mark_used();

            // Quick health check for connections that haven't been used recently
            if pooled_conn.last_used.elapsed() > HEALTH_CHECK_INTERVAL
                && !pooled_conn.health_check().await
            {
                // Connection is unhealthy, create a new one
                drop(connections); // Release lock before creating new connection
                return self
                    .create_connection_with_retry()
                    .await
                    .map(|conn| conn.connection);
            }

            let connection = pooled_conn.connection.clone();
            connections.push(pooled_conn); // Return to pool
            return Ok(connection);
        }

        // No healthy connections available, create new one if under max size
        if connections.len() < self.max_size {
            drop(connections); // Release lock before creating new connection
            return self
                .create_connection_with_retry()
                .await
                .map(|conn| conn.connection);
        }

        // Pool is full, return the oldest connection
        if let Some(mut pooled_conn) = connections.pop() {
            pooled_conn.mark_used();
            let connection = pooled_conn.connection.clone();
            connections.push(pooled_conn);
            Ok(connection)
        } else {
            // Should not happen, but fallback to creating new connection
            drop(connections);
            self.create_connection_with_retry()
                .await
                .map(|conn| conn.connection)
        }
    }

    async fn create_connection_with_retry(&self) -> Result<PooledConnection, BrokerError> {
        let mut attempt = 0;
        let mut backoff = INITIAL_BACKOFF;

        while attempt < MAX_RETRY_ATTEMPTS {
            match self.create_connection().await {
                Ok(conn) => return Ok(conn),
                Err(e) if attempt == MAX_RETRY_ATTEMPTS - 1 => return Err(e),
                Err(_) => {
                    attempt += 1;
                    sleep(backoff).await;
                    backoff = std::cmp::min(backoff * 2, Duration::from_secs(5));
                }
            }
        }

        Err(BrokerError::ConnectionError(
            "Failed to create connection after retries".to_string(),
        ))
    }

    #[allow(dead_code)]
    pub async fn return_connection(&self, connection: MultiplexedConnection) {
        let mut connections = self.connections.lock().await;
        if connections.len() < self.max_size {
            connections.push(PooledConnection::new(connection));
        }
        // If pool is full, just drop the connection
    }

    pub async fn health_check(&self) -> Result<(), BrokerError> {
        let mut connections = self.connections.lock().await;

        // Check health of all pooled connections
        for conn in connections.iter_mut() {
            if !conn.health_check().await {
                // Mark as unhealthy - it will be replaced on next use
                conn.is_healthy = false;
            }
        }

        // Remove unhealthy connections
        connections.retain(|conn| conn.is_healthy);

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn close(&self) {
        let mut connections = self.connections.lock().await;
        connections.clear();
    }
}

impl Drop for ConnectionPool {
    fn drop(&mut self) {
        // Note: Cannot run async code in Drop, so connections will be cleaned up
        // when they go out of scope. For proper cleanup, call close() explicitly.
    }
}
