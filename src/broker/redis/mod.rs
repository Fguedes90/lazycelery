pub mod connection;
pub mod facade;
pub mod operations;
pub mod pool;
pub mod protocol;

use crate::broker::Broker;
use crate::error::BrokerError;
use crate::models::{Queue, Task, Worker};
use async_trait::async_trait;
use tracing::{debug, info};

// Re-export for backward compatibility
pub use facade::BrokerFacade;

/// Redis broker implementation using the improved facade pattern
pub struct RedisBroker {
    facade: BrokerFacade,
}

#[async_trait]
impl Broker for RedisBroker {
    async fn connect(url: &str) -> Result<Self, BrokerError> {
        info!("Connecting to Redis broker using facade pattern");
        debug!("Redis URL: {}", url.split('@').next_back().unwrap_or("hidden"));

        let facade = BrokerFacade::new(url).await?;

        // Perform initial health check
        facade.health_check().await?;

        info!("Redis broker connected successfully");

        Ok(Self { facade })
    }

    async fn get_workers(&self) -> Result<Vec<Worker>, BrokerError> {
        self.facade.get_workers().await
    }

    async fn get_tasks(&self) -> Result<Vec<Task>, BrokerError> {
        self.facade.get_tasks().await
    }

    async fn get_queues(&self) -> Result<Vec<Queue>, BrokerError> {
        self.facade.get_queues().await
    }

    async fn retry_task(&self, task_id: &str) -> Result<(), BrokerError> {
        self.facade.retry_task(task_id).await
    }

    async fn revoke_task(&self, task_id: &str) -> Result<(), BrokerError> {
        self.facade.revoke_task(task_id).await
    }

    async fn purge_queue(&self, queue_name: &str) -> Result<u64, BrokerError> {
        self.facade.purge_queue(queue_name).await
    }
}
