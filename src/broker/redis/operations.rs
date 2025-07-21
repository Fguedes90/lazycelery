use crate::error::BrokerError;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use serde_json::Value;

/// Input validation utilities for Redis operations
mod validation {
    use crate::error::BrokerError;

    /// Maximum allowed length for task IDs (based on Celery UUID format)
    const MAX_TASK_ID_LENGTH: usize = 36;

    /// Maximum allowed length for queue names
    const MAX_QUEUE_NAME_LENGTH: usize = 255;

    /// Valid characters for task IDs (UUID format)
    const TASK_ID_CHARS: &str = "abcdefABCDEF0123456789-";

    /// Valid characters for queue names (alphanumeric, dots, underscores, hyphens)
    const QUEUE_NAME_CHARS: &str =
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789._-";

    /// Validate task ID format and content
    pub fn validate_task_id(task_id: &str) -> Result<(), BrokerError> {
        if task_id.is_empty() {
            return Err(BrokerError::ValidationError(
                "Task ID cannot be empty".to_string(),
            ));
        }

        if task_id.len() > MAX_TASK_ID_LENGTH {
            return Err(BrokerError::ValidationError(format!(
                "Task ID exceeds maximum length of {MAX_TASK_ID_LENGTH} characters"
            )));
        }

        // Check for valid UUID-like format (8-4-4-4-12 pattern)
        if task_id.len() == 36 {
            let parts: Vec<&str> = task_id.split('-').collect();
            if parts.len() != 5
                || parts[0].len() != 8
                || parts[1].len() != 4
                || parts[2].len() != 4
                || parts[3].len() != 4
                || parts[4].len() != 12
            {
                return Err(BrokerError::ValidationError(
                    "Task ID must be in valid UUID format (8-4-4-4-12)".to_string(),
                ));
            }
        }

        // Validate characters
        for ch in task_id.chars() {
            if !TASK_ID_CHARS.contains(ch) {
                return Err(BrokerError::ValidationError(format!(
                    "Task ID contains invalid character: '{ch}'"
                )));
            }
        }

        Ok(())
    }

    /// Validate queue name format and content
    pub fn validate_queue_name(queue_name: &str) -> Result<(), BrokerError> {
        if queue_name.is_empty() {
            return Err(BrokerError::ValidationError(
                "Queue name cannot be empty".to_string(),
            ));
        }

        if queue_name.len() > MAX_QUEUE_NAME_LENGTH {
            return Err(BrokerError::ValidationError(format!(
                "Queue name exceeds maximum length of {MAX_QUEUE_NAME_LENGTH} characters"
            )));
        }

        // Validate characters
        for ch in queue_name.chars() {
            if !QUEUE_NAME_CHARS.contains(ch) {
                return Err(BrokerError::ValidationError(format!(
                    "Queue name contains invalid character: '{ch}'"
                )));
            }
        }

        // Additional security checks
        if queue_name.starts_with('.') || queue_name.ends_with('.') {
            return Err(BrokerError::ValidationError(
                "Queue name cannot start or end with a dot".to_string(),
            ));
        }

        if queue_name.contains("..") {
            return Err(BrokerError::ValidationError(
                "Queue name cannot contain consecutive dots".to_string(),
            ));
        }

        Ok(())
    }

    /// Sanitize Redis key to prevent injection attacks
    pub fn sanitize_redis_key(key: &str) -> Result<String, BrokerError> {
        if key.is_empty() {
            return Err(BrokerError::ValidationError(
                "Redis key cannot be empty".to_string(),
            ));
        }

        if key.len() > 512 {
            return Err(BrokerError::ValidationError(
                "Redis key exceeds maximum length of 512 characters".to_string(),
            ));
        }

        // Check for dangerous patterns
        let dangerous_patterns = [
            "EVAL",
            "SCRIPT",
            "FLUSHALL",
            "FLUSHDB",
            "CONFIG",
            "SHUTDOWN",
            "DEBUG",
            "SAVE",
            "BGSAVE",
            "BGREWRITEAOF",
            "LASTSAVE",
        ];

        let key_upper = key.to_uppercase();
        for pattern in &dangerous_patterns {
            if key_upper.contains(pattern) {
                return Err(BrokerError::ValidationError(format!(
                    "Redis key contains dangerous pattern: {pattern}"
                )));
            }
        }

        // Only allow safe characters in keys
        const SAFE_KEY_CHARS: &str =
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789._:-";
        for ch in key.chars() {
            if !SAFE_KEY_CHARS.contains(ch) {
                return Err(BrokerError::ValidationError(format!(
                    "Redis key contains unsafe character: '{ch}'"
                )));
            }
        }

        Ok(key.to_string())
    }
}

pub struct TaskOperations;

impl TaskOperations {
    pub async fn retry_task(
        connection: &MultiplexedConnection,
        task_id: &str,
    ) -> Result<(), BrokerError> {
        // Validate input
        validation::validate_task_id(task_id)?;

        let mut conn = connection.clone();

        // Get the task metadata to extract task information
        let task_key = validation::sanitize_redis_key(&format!("celery-task-meta-{task_id}"))?;
        let task_data: Option<String> = conn
            .get(&task_key)
            .await
            .map_err(|e| BrokerError::OperationError(e.to_string()))?;

        let task_data = task_data
            .ok_or_else(|| BrokerError::OperationError(format!("Task {task_id} not found")))?;

        let task_json: Value = serde_json::from_str(&task_data)
            .map_err(|e| BrokerError::OperationError(e.to_string()))?;

        // Only retry failed tasks
        let status = task_json
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("");
        if status != "FAILURE" {
            return Err(BrokerError::OperationError(format!(
                "Can only retry failed tasks, task {task_id} is {status}"
            )));
        }

        // For a proper retry, we would need the original task message with args/kwargs
        // Since we only have the result metadata, we'll update the status to indicate retry
        let mut updated_task = task_json.clone();
        updated_task["status"] = Value::String("RETRY".to_string());
        updated_task["retries"] = Value::Number(
            (task_json
                .get("retries")
                .and_then(|r| r.as_i64())
                .unwrap_or(0)
                + 1)
            .into(),
        );

        // Update the task metadata
        let updated_data = serde_json::to_string(&updated_task)
            .map_err(|e| BrokerError::OperationError(e.to_string()))?;

        conn.set::<_, _, ()>(&task_key, updated_data)
            .await
            .map_err(|e| BrokerError::OperationError(e.to_string()))?;

        // Note: In a real implementation, we would republish the original task message
        // to the appropriate queue, but that requires storing the original message

        Ok(())
    }

    pub async fn revoke_task(
        connection: &MultiplexedConnection,
        task_id: &str,
    ) -> Result<(), BrokerError> {
        // Validate input
        validation::validate_task_id(task_id)?;

        let mut conn = connection.clone();

        // Add task to Celery's revoked tasks set
        let revoked_key = validation::sanitize_redis_key("revoked")?;
        conn.sadd::<_, _, ()>(&revoked_key, task_id)
            .await
            .map_err(|e| BrokerError::OperationError(e.to_string()))?;

        // Update task metadata if it exists
        let task_key = validation::sanitize_redis_key(&format!("celery-task-meta-{task_id}"))?;
        if let Ok(Some(task_data)) = conn.get::<_, Option<String>>(&task_key).await {
            if let Ok(mut task_json) = serde_json::from_str::<Value>(&task_data) {
                // Update status to revoked
                task_json["status"] = Value::String("REVOKED".to_string());

                if let Ok(updated_data) = serde_json::to_string(&task_json) {
                    let _: Result<(), _> = conn.set(&task_key, updated_data).await;
                }
            }
        }

        // Note: In a real implementation with active workers, the workers would
        // check the revoked set and terminate any running tasks with this ID

        Ok(())
    }

    pub async fn purge_queue(
        connection: &MultiplexedConnection,
        queue_name: &str,
    ) -> Result<u64, BrokerError> {
        // Validate input
        validation::validate_queue_name(queue_name)?;
        let sanitized_queue = validation::sanitize_redis_key(queue_name)?;

        let mut conn = connection.clone();

        // Get current queue length for reporting
        let queue_length: u64 = conn
            .llen(&sanitized_queue)
            .await
            .map_err(|e| BrokerError::OperationError(e.to_string()))?;

        // Delete all messages from the queue (Redis LIST)
        // Using DEL command to completely remove the list
        let deleted: u64 = conn
            .del(&sanitized_queue)
            .await
            .map_err(|e| BrokerError::OperationError(e.to_string()))?;

        // Return the number of messages that were purged
        if deleted > 0 {
            Ok(queue_length)
        } else {
            Ok(0)
        }
    }
}
