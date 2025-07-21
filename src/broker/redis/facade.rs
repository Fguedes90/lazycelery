use crate::broker::redis::operations::TaskOperations;
use crate::broker::redis::pool::ConnectionPool;
use crate::broker::redis::protocol::ProtocolParser;
use crate::error::BrokerError;
use crate::models::{Queue, Task, Worker};
use std::sync::Arc;
use tracing::{debug, error, info, instrument, warn};

/// BrokerFacade provides a clean, high-level interface for Redis broker operations.
/// It encapsulates connection management, error handling, and operation complexity.
pub struct BrokerFacade {
    pool: Arc<ConnectionPool>,
}

impl BrokerFacade {
    pub async fn new(url: &str) -> Result<Self, BrokerError> {
        info!(
            "Creating new Redis broker facade for URL: {}",
            url.split('@').next_back().unwrap_or("hidden")
        );

        let pool = ConnectionPool::new(url, Some(10)).await.map_err(|e| {
            error!("Failed to create connection pool: {}", e);
            e
        })?;

        info!("Redis broker facade created successfully");

        Ok(Self {
            pool: Arc::new(pool),
        })
    }

    /// Get all workers with comprehensive error handling and logging
    #[instrument(skip(self), name = "get_workers")]
    pub async fn get_workers(&self) -> Result<Vec<Worker>, BrokerError> {
        debug!("Fetching workers from Redis");

        let connection = self.get_pooled_connection("get_workers").await?;

        match ProtocolParser::parse_workers(&connection).await {
            Ok(workers) => {
                info!("Successfully retrieved {} workers", workers.len());
                debug!(
                    "Workers: {:?}",
                    workers.iter().map(|w| &w.hostname).collect::<Vec<_>>()
                );
                Ok(workers)
            }
            Err(e) => {
                error!("Failed to parse workers: {}", e);
                Err(self.add_operation_context(e, "get_workers"))
            }
        }
    }

    /// Get all tasks with comprehensive error handling and logging
    #[instrument(skip(self), name = "get_tasks")]
    pub async fn get_tasks(&self) -> Result<Vec<Task>, BrokerError> {
        debug!("Fetching tasks from Redis");

        let connection = self.get_pooled_connection("get_tasks").await?;

        match ProtocolParser::parse_tasks(&connection).await {
            Ok(tasks) => {
                info!("Successfully retrieved {} tasks", tasks.len());
                debug!(
                    "Task statuses: {:?}",
                    tasks.iter().map(|t| &t.status).collect::<Vec<_>>()
                );
                Ok(tasks)
            }
            Err(e) => {
                error!("Failed to parse tasks: {}", e);
                Err(self.add_operation_context(e, "get_tasks"))
            }
        }
    }

    /// Get all queues with comprehensive error handling and logging
    #[instrument(skip(self), name = "get_queues")]
    pub async fn get_queues(&self) -> Result<Vec<Queue>, BrokerError> {
        debug!("Fetching queues from Redis");

        let connection = self.get_pooled_connection("get_queues").await?;

        match ProtocolParser::parse_queues(&connection).await {
            Ok(queues) => {
                info!("Successfully retrieved {} queues", queues.len());
                debug!(
                    "Queue names: {:?}",
                    queues.iter().map(|q| &q.name).collect::<Vec<_>>()
                );
                Ok(queues)
            }
            Err(e) => {
                error!("Failed to parse queues: {}", e);
                Err(self.add_operation_context(e, "get_queues"))
            }
        }
    }

    /// Retry a task with validation and comprehensive error handling
    #[instrument(skip(self), fields(task_id = %task_id), name = "retry_task")]
    pub async fn retry_task(&self, task_id: &str) -> Result<(), BrokerError> {
        info!("Retrying task: {}", task_id);

        if task_id.is_empty() {
            warn!("Empty task ID provided for retry operation");
            return Err(BrokerError::OperationError(
                "Task ID cannot be empty".to_string(),
            ));
        }

        let connection = self.get_pooled_connection("retry_task").await?;

        match TaskOperations::retry_task(&connection, task_id).await {
            Ok(()) => {
                info!("Successfully retried task: {}", task_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to retry task {}: {}", task_id, e);
                Err(self.add_operation_context(e, "retry_task"))
            }
        }
    }

    /// Revoke a task with validation and comprehensive error handling
    #[instrument(skip(self), fields(task_id = %task_id), name = "revoke_task")]
    pub async fn revoke_task(&self, task_id: &str) -> Result<(), BrokerError> {
        info!("Revoking task: {}", task_id);

        if task_id.is_empty() {
            warn!("Empty task ID provided for revoke operation");
            return Err(BrokerError::OperationError(
                "Task ID cannot be empty".to_string(),
            ));
        }

        let connection = self.get_pooled_connection("revoke_task").await?;

        match TaskOperations::revoke_task(&connection, task_id).await {
            Ok(()) => {
                info!("Successfully revoked task: {}", task_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to revoke task {}: {}", task_id, e);
                Err(self.add_operation_context(e, "revoke_task"))
            }
        }
    }

    /// Purge a queue with validation and comprehensive error handling
    #[instrument(skip(self), fields(queue_name = %queue_name), name = "purge_queue")]
    pub async fn purge_queue(&self, queue_name: &str) -> Result<u64, BrokerError> {
        info!("Purging queue: {}", queue_name);

        if queue_name.is_empty() {
            warn!("Empty queue name provided for purge operation");
            return Err(BrokerError::OperationError(
                "Queue name cannot be empty".to_string(),
            ));
        }

        let connection = self.get_pooled_connection("purge_queue").await?;

        match TaskOperations::purge_queue(&connection, queue_name).await {
            Ok(purged_count) => {
                info!(
                    "Successfully purged {} messages from queue: {}",
                    purged_count, queue_name
                );
                Ok(purged_count)
            }
            Err(e) => {
                error!("Failed to purge queue {}: {}", queue_name, e);
                Err(self.add_operation_context(e, "purge_queue"))
            }
        }
    }

    /// Perform health check on the connection pool
    #[instrument(skip(self), name = "health_check")]
    pub async fn health_check(&self) -> Result<(), BrokerError> {
        debug!("Performing health check on connection pool");

        match self.pool.health_check().await {
            Ok(()) => {
                debug!("Health check passed");
                Ok(())
            }
            Err(e) => {
                warn!("Health check failed: {}", e);
                Err(self.add_operation_context(e, "health_check"))
            }
        }
    }

    /// Get statistics about the connection pool
    pub async fn get_pool_stats(&self) -> PoolStats {
        // This is a simplified implementation - in a real scenario,
        // we'd track more detailed statistics
        PoolStats {
            active_connections: 1,  // Simplified
            total_connections: 1,   // Simplified
            healthy_connections: 1, // Simplified
        }
    }

    /// Internal method to get a connection from the pool with context
    async fn get_pooled_connection(
        &self,
        operation: &str,
    ) -> Result<redis::aio::MultiplexedConnection, BrokerError> {
        debug!("Getting pooled connection for operation: {}", operation);

        self.pool.get_connection().await.map_err(|e| {
            error!("Failed to get pooled connection for {}: {}", operation, e);
            self.add_operation_context(e, operation)
        })
    }

    /// Add contextual information to errors for better debugging
    fn add_operation_context(&self, error: BrokerError, operation: &str) -> BrokerError {
        match error {
            BrokerError::ConnectionError(msg) => {
                BrokerError::ConnectionError(format!("Operation '{operation}': {msg}"))
            }
            BrokerError::OperationError(msg) => {
                BrokerError::OperationError(format!("Operation '{operation}': {msg}"))
            }
            other => other, // Don't modify other error types
        }
    }
}

/// Statistics about the connection pool
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub active_connections: usize,
    pub total_connections: usize,
    pub healthy_connections: usize,
}

impl Drop for BrokerFacade {
    fn drop(&mut self) {
        debug!("BrokerFacade being dropped");
        // Pool cleanup will happen automatically when Arc is dropped
    }
}
