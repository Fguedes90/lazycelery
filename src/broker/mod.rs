pub mod redis;
pub mod amqp;

use async_trait::async_trait;
use crate::models::{Worker, Task, Queue};
use crate::error::BrokerError;

#[async_trait]
#[allow(dead_code)]
pub trait Broker: Send + Sync {
    async fn connect(url: &str) -> Result<Self, BrokerError>
    where
        Self: Sized;

    async fn get_workers(&self) -> Result<Vec<Worker>, BrokerError>;
    async fn get_tasks(&self) -> Result<Vec<Task>, BrokerError>;
    async fn get_queues(&self) -> Result<Vec<Queue>, BrokerError>;
    async fn retry_task(&self, task_id: &str) -> Result<(), BrokerError>;
    async fn revoke_task(&self, task_id: &str) -> Result<(), BrokerError>;
}
