//! Task parser for Redis Celery protocol
//!
//! This module handles parsing task information from Redis data structures.
//! It extracts task metadata, status, and combines information from both
//! completed tasks (metadata) and pending tasks (queue messages).

use crate::error::BrokerError;
use crate::models::{Task, TaskStatus};
use base64::Engine;
use chrono::{DateTime, Utc};
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use serde_json::Value;
use std::collections::HashMap;

// Configuration constants for task parsing
const MAX_TASK_RESULTS: usize = 100;
const MAX_QUEUE_MESSAGES: usize = 100;
const MAX_PENDING_TASKS: usize = 20;

/// Parser for task-related data from Redis
pub struct TaskParser;

impl TaskParser {
    /// Parse tasks from Redis connection
    ///
    /// Combines information from task metadata (completed tasks) and queue messages
    /// (pending tasks) to provide a comprehensive view of all tasks.
    pub async fn parse_tasks(connection: &MultiplexedConnection) -> Result<Vec<Task>, BrokerError> {
        let mut conn = connection.clone();
        let mut tasks = Vec::new();

        // First, get task names from pending queue messages
        let task_names = Self::get_queue_messages(&mut conn).await?;

        // Get task results from metadata keys
        Self::parse_task_metadata(&mut conn, &mut tasks, &task_names).await?;

        // Add pending tasks from queues that might not have metadata yet
        Self::add_pending_tasks_from_queues(&mut conn, &mut tasks).await?;

        Ok(tasks)
    }

    /// Extract task names and IDs from queue messages
    ///
    /// Scans common queues to build a mapping of task IDs to task names,
    /// which helps identify task types for completed tasks that may not
    /// have this information in their metadata.
    async fn get_queue_messages(
        conn: &mut MultiplexedConnection,
    ) -> Result<HashMap<String, String>, BrokerError> {
        let mut task_names: HashMap<String, String> = HashMap::new();
        let queue_names = vec!["celery", "default", "priority"];

        for queue_name in &queue_names {
            match conn.llen::<_, u64>(queue_name).await {
                Ok(queue_length) if queue_length > 0 => {
                    match conn
                        .lrange::<_, Vec<String>>(queue_name, 0, MAX_QUEUE_MESSAGES as isize)
                        .await
                    {
                        Ok(messages) => {
                            for message in &messages {
                                if let Ok(task_message) = serde_json::from_str::<Value>(message) {
                                    if let Some(headers) = task_message.get("headers") {
                                        if let (Some(task_id), Some(task_name)) = (
                                            headers.get("id").and_then(|id| id.as_str()),
                                            headers.get("task").and_then(|task| task.as_str()),
                                        ) {
                                            task_names
                                                .insert(task_id.to_string(), task_name.to_string());
                                        }
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            // Skip queue if we can't read messages - continue with other queues
                            continue;
                        }
                    }
                }
                _ => {
                    // Skip empty or inaccessible queues
                    continue;
                }
            }
        }

        Ok(task_names)
    }

    /// Parse task metadata from Redis keys
    ///
    /// Processes completed task metadata stored in Redis to extract task
    /// information including status, results, and execution details.
    async fn parse_task_metadata(
        conn: &mut MultiplexedConnection,
        tasks: &mut Vec<Task>,
        task_names: &HashMap<String, String>,
    ) -> Result<(), BrokerError> {
        let task_keys: Vec<String> = conn.keys("celery-task-meta-*").await.map_err(|e| {
            BrokerError::OperationError(format!("Failed to get task metadata keys: {e}"))
        })?;

        for key in task_keys.iter().take(MAX_TASK_RESULTS) {
            match conn.get::<_, String>(key).await {
                Ok(data) => {
                    match serde_json::from_str::<Value>(&data) {
                        Ok(task_data) => {
                            match Self::extract_task_from_metadata(key, &task_data, task_names) {
                                Ok(task) => tasks.push(task),
                                Err(_) => {
                                    // Skip malformed task metadata - continue processing
                                    continue;
                                }
                            }
                        }
                        Err(_) => {
                            // Skip malformed JSON - continue processing
                            continue;
                        }
                    }
                }
                Err(_) => {
                    // Skip inaccessible keys - continue processing
                    continue;
                }
            }
        }

        Ok(())
    }

    /// Extract task information from metadata
    ///
    /// Converts raw task metadata into a Task struct with all relevant
    /// information including status, results, and timing data.
    fn extract_task_from_metadata(
        key: &str,
        task_data: &Value,
        task_names: &HashMap<String, String>,
    ) -> Result<Task, BrokerError> {
        let task_id = key
            .strip_prefix("celery-task-meta-")
            .unwrap_or("unknown")
            .to_string();

        let timestamp = Self::parse_timestamp(task_data);
        let task_name = Self::get_task_name(&task_id, task_data, task_names);
        let status = Self::parse_task_status(task_data);

        Ok(Task {
            id: task_id,
            name: task_name,
            args: task_data
                .get("args")
                .map(|a| a.to_string())
                .unwrap_or_else(|| "[]".to_string()),
            kwargs: task_data
                .get("kwargs")
                .map(|k| k.to_string())
                .unwrap_or_else(|| "{}".to_string()),
            status,
            worker: None, // Task metadata doesn't contain worker hostname
            timestamp,
            result: task_data.get("result").map(|r| r.to_string()),
            traceback: task_data
                .get("traceback")
                .and_then(|t| t.as_str())
                .map(|s| s.to_string()),
        })
    }

    /// Parse timestamp from task data
    ///
    /// Extracts and parses the completion timestamp from task metadata,
    /// using the current time as fallback if parsing fails.
    fn parse_timestamp(task_data: &Value) -> DateTime<Utc> {
        if let Some(date_done) = task_data.get("date_done").and_then(|d| d.as_str()) {
            date_done
                .parse::<DateTime<Utc>>()
                .unwrap_or_else(|_| Utc::now())
        } else {
            Utc::now()
        }
    }

    /// Get task name from various sources
    ///
    /// Attempts to determine the task name from the task names mapping
    /// (from queue messages) or task metadata, with fallback to "unknown".
    fn get_task_name(
        task_id: &str,
        task_data: &Value,
        task_names: &HashMap<String, String>,
    ) -> String {
        task_names
            .get(task_id)
            .cloned()
            .or_else(|| {
                task_data
                    .get("task")
                    .and_then(|t| t.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// Parse task status from metadata
    ///
    /// Converts string status values from Celery into TaskStatus enum values.
    fn parse_task_status(task_data: &Value) -> TaskStatus {
        match task_data.get("status").and_then(|s| s.as_str()) {
            Some("SUCCESS") => TaskStatus::Success,
            Some("FAILURE") => TaskStatus::Failure,
            Some("PENDING") => TaskStatus::Pending,
            Some("RETRY") => TaskStatus::Retry,
            Some("REVOKED") => TaskStatus::Revoked,
            Some("STARTED") => TaskStatus::Active,
            _ => TaskStatus::Pending,
        }
    }

    /// Add pending tasks from queue messages
    ///
    /// Scans queues for pending tasks that may not have metadata yet
    /// and adds them to the task list with PENDING status.
    async fn add_pending_tasks_from_queues(
        conn: &mut MultiplexedConnection,
        tasks: &mut Vec<Task>,
    ) -> Result<(), BrokerError> {
        let queue_names = vec!["celery", "default", "priority"];

        for queue_name in &queue_names {
            match conn.llen::<_, u64>(queue_name).await {
                Ok(queue_length) if queue_length > 0 => {
                    match conn
                        .lrange::<_, Vec<String>>(queue_name, 0, MAX_PENDING_TASKS as isize)
                        .await
                    {
                        Ok(messages) => {
                            for message in &messages {
                                if let Ok(task_message) = serde_json::from_str::<Value>(message) {
                                    match Self::parse_task_message(&task_message, tasks) {
                                        Ok(Some(task)) => tasks.push(task),
                                        Ok(None) => continue, // Task already exists or invalid
                                        Err(_) => continue,   // Skip malformed message
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            // Skip queue if we can't read messages
                            continue;
                        }
                    }
                }
                _ => {
                    // Skip empty or inaccessible queues
                    continue;
                }
            }
        }

        Ok(())
    }

    /// Parse task from queue message
    ///
    /// Extracts task information from a queue message, checking if the task
    /// already exists to avoid duplicates.
    fn parse_task_message(
        task_message: &Value,
        existing_tasks: &[Task],
    ) -> Result<Option<Task>, BrokerError> {
        if let Some(headers) = task_message.get("headers") {
            if let (Some(task_id), Some(task_name)) = (
                headers.get("id").and_then(|id| id.as_str()),
                headers.get("task").and_then(|task| task.as_str()),
            ) {
                // Only add if not already in our task list
                if !existing_tasks.iter().any(|t| t.id == task_id) {
                    let (args, kwargs) = Self::decode_task_body(task_message);

                    return Ok(Some(Task {
                        id: task_id.to_string(),
                        name: task_name.to_string(),
                        args,
                        kwargs,
                        status: TaskStatus::Pending,
                        worker: None,
                        timestamp: Utc::now(),
                        result: None,
                        traceback: None,
                    }));
                }
            }
        }

        Ok(None)
    }

    /// Decode base64-encoded task body
    ///
    /// Attempts to decode the task body from base64 and extract
    /// arguments and keyword arguments from the Celery message format.
    fn decode_task_body(task_message: &Value) -> (String, String) {
        if let Some(body) = task_message.get("body").and_then(|b| b.as_str()) {
            if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(body) {
                if let Ok(body_str) = String::from_utf8(decoded) {
                    if let Ok(body_json) = serde_json::from_str::<Value>(&body_str) {
                        let args = body_json
                            .get(0)
                            .map(|a| a.to_string())
                            .unwrap_or_else(|| "[]".to_string());
                        let kwargs = body_json
                            .get(1)
                            .map(|k| k.to_string())
                            .unwrap_or_else(|| "{}".to_string());
                        return (args, kwargs);
                    }
                }
            }
        }

        ("[]".to_string(), "{}".to_string())
    }
}
