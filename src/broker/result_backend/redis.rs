//! Redis result backend implementation
//!
//! This module provides Redis as a result backend for storing task results.

use async_trait::async_trait;
use redis::{AsyncCommands, Client};
use tracing::{debug, error};

use crate::broker::ResultBackend;
use crate::error::BrokerError;
use crate::models::{Task, TaskStatus};

/// Redis result backend
pub struct RedisResultBackend {
    client: Client,
}

impl RedisResultBackend {
    /// Create a new Redis result backend
    pub async fn connect(url: &str) -> Result<Self, BrokerError> {
        let client = Client::open(url)
            .map_err(|e| BrokerError::ConnectionError(format!("Failed to connect to Redis: {}", e)))?;

        // Test the connection
        let mut conn = client
            .get_multiplexed_tokio_connection()
            .await
            .map_err(|e| BrokerError::ConnectionError(format!("Failed to get connection: {}", e)))?;

        redis::cmd("PING")
            .query_async::<_, String>(&mut conn)
            .await
            .map_err(|e| BrokerError::ConnectionError(format!("Redis ping failed: {}", e)))?;

        tracing::info!("Redis result backend connected");

        Ok(Self { client })
    }

    /// Parse a task from Redis result metadata
    fn parse_task_meta(task_id: &str, data: &str) -> Result<Task, BrokerError> {
        let json: serde_json::Value = serde_json::from_str(data)
            .map_err(|e| BrokerError::OperationError(format!("Failed to parse task result: {}", e)))?;

        let status = match json.get("status").and_then(|v| v.as_str()) {
            Some("pending") => TaskStatus::Pending,
            Some("started") => TaskStatus::Active,
            Some("success") => TaskStatus::Success,
            Some("failure") => TaskStatus::Failure,
            Some("retry") => TaskStatus::Retry,
            Some("revoked") => TaskStatus::Revoked,
            _ => TaskStatus::Pending,
        };

        let result = json.get("result").and_then(|v| v.as_str()).map(String::from);
        let traceback = json.get("traceback").and_then(|v| v.as_str()).map(String::from);

        let name = json
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let args = json
            .get("args")
            .and_then(|v| v.as_str())
            .unwrap_or("[]")
            .to_string();

        let kwargs = json
            .get("kwargs")
            .and_then(|v| v.as_str())
            .unwrap_or("{}")
            .to_string();

        // Try to get timestamps
        let timestamp = chrono::Utc::now();
        
        // Try to get worker info
        let worker = json
            .get("worker")
            .and_then(|v| v.as_str())
            .map(String::from);

        Ok(Task {
            id: task_id.to_string(),
            name,
            args,
            kwargs,
            status,
            worker,
            timestamp,
            result,
            traceback,
        })
    }
}

#[async_trait]
impl ResultBackend for RedisResultBackend {
    async fn get_task_result(&self, task_id: &str) -> Result<Option<Task>, BrokerError> {
        let mut conn = self
            .client
            .get_multiplexed_tokio_connection()
            .await
            .map_err(|e| BrokerError::OperationError(format!("Failed to get connection: {}", e)))?;

        // Celery stores task results with this key pattern
        let key = format!("celery-task-meta-{}", task_id);

        debug!("Getting task result for: {}", key);

        let result: Option<String> = conn
            .get(&key)
            .await
            .map_err(|e| {
                error!("Failed to get task result: {}", e);
                BrokerError::OperationError(format!("Failed to get task result: {}", e))
            })?;

        match result {
            Some(data) => {
                let task = Self::parse_task_meta(task_id, &data)?;
                Ok(Some(task))
            }
            None => Ok(None),
        }
    }

    async fn connect(url: &str) -> Result<Self, BrokerError>
    where
        Self: Sized,
    {
        Self::connect(url).await
    }
}
