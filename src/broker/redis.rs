use crate::broker::Broker;
use crate::error::BrokerError;
use crate::models::{Queue, Task, TaskStatus, Worker, WorkerStatus};
use async_trait::async_trait;
use base64::Engine;
use chrono::{DateTime, Utc};
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client};
use serde_json::Value;
use std::collections::HashMap;

pub struct RedisBroker {
    _client: Client,
    connection: MultiplexedConnection,
}

impl RedisBroker {
    async fn parse_workers(&self) -> Result<Vec<Worker>, BrokerError> {
        let mut conn = self.connection.clone();

        // Analyze both task metadata and queue messages to build worker information
        let mut worker_stats: HashMap<String, (u64, u64, Vec<String>)> = HashMap::new(); // (processed, failed, queues)
        let active_workers: HashMap<String, Vec<String>> = HashMap::new(); // worker -> active tasks

        // 1. Check task metadata for completed tasks
        let task_keys: Vec<String> = conn
            .keys("celery-task-meta-*")
            .await
            .map_err(|e| BrokerError::OperationError(e.to_string()))?;

        for key in task_keys.iter().take(500) {
            if let Ok(data) = conn.get::<_, String>(key).await {
                if let Ok(task_data) = serde_json::from_str::<Value>(&data) {
                    let status = task_data
                        .get("status")
                        .and_then(|s| s.as_str())
                        .unwrap_or("UNKNOWN");

                    // For completed tasks, we don't have hostname in metadata
                    // So we'll create a generic worker based on activity
                    let hostname = "celery-worker".to_string();
                    let (processed, failed, queues) =
                        worker_stats.entry(hostname).or_insert((0, 0, Vec::new()));

                    match status {
                        "SUCCESS" => *processed += 1,
                        "FAILURE" => *failed += 1,
                        _ => {}
                    }

                    // Add default queue
                    if !queues.contains(&"celery".to_string()) {
                        queues.push("celery".to_string());
                    }
                }
            }
        }

        // 2. Check pending tasks in queues for worker origin information
        let queue_names = vec!["celery", "default", "priority"];
        for queue_name in queue_names {
            let queue_length: u64 = conn.llen(queue_name).await.unwrap_or(0);
            if queue_length > 0 {
                // Get first few messages to extract worker information
                let messages: Vec<String> = conn.lrange(queue_name, 0, 5).await.unwrap_or_default();

                for message in messages {
                    if let Ok(task_message) = serde_json::from_str::<Value>(&message) {
                        if let Some(headers) = task_message.get("headers") {
                            if let Some(origin) = headers.get("origin").and_then(|o| o.as_str()) {
                                // Extract hostname from origin like "gen447152@archflowx13"
                                let hostname = if let Some(at_pos) = origin.find('@') {
                                    origin[at_pos + 1..].to_string()
                                } else {
                                    origin.to_string()
                                };

                                let (_processed, _failed, queues) =
                                    worker_stats.entry(hostname).or_insert((0, 0, Vec::new()));
                                if !queues.contains(&queue_name.to_string()) {
                                    queues.push(queue_name.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        // 3. Build worker list
        let mut workers = Vec::new();
        for (hostname, (processed, failed, queues)) in worker_stats {
            let active_tasks = active_workers.get(&hostname).cloned().unwrap_or_default();

            // Determine worker status - if we have recent task data, assume online
            let status = if processed > 0 || failed > 0 {
                WorkerStatus::Online
            } else {
                WorkerStatus::Offline
            };

            workers.push(Worker {
                hostname,
                status,
                concurrency: 16, // Default concurrency
                queues: if queues.is_empty() {
                    vec!["celery".to_string()]
                } else {
                    queues
                },
                active_tasks,
                processed,
                failed,
            });
        }

        // If no workers found, check if there are any tasks or queues indicating workers should exist
        if workers.is_empty() {
            let celery_queue_len: u64 = conn.llen("celery").await.unwrap_or(0);
            let task_count = task_keys.len();

            if celery_queue_len > 0 || task_count > 0 {
                // There is activity, so assume a worker exists
                workers.push(Worker {
                    hostname: "detected-worker".to_string(),
                    status: if celery_queue_len > 0 {
                        WorkerStatus::Offline
                    } else {
                        WorkerStatus::Online
                    },
                    concurrency: 16,
                    queues: vec!["celery".to_string()],
                    active_tasks: vec![],
                    processed: task_count as u64,
                    failed: 0,
                });
            }
        }

        Ok(workers)
    }

    async fn parse_tasks(&self) -> Result<Vec<Task>, BrokerError> {
        let mut conn = self.connection.clone();
        let mut tasks = Vec::new();

        // First, get task names from pending queue messages
        let mut task_names: HashMap<String, String> = HashMap::new();
        let queue_names = vec!["celery", "default", "priority"];

        for queue_name in &queue_names {
            let queue_length: u64 = conn.llen(queue_name).await.unwrap_or(0);
            if queue_length > 0 {
                let messages: Vec<String> =
                    conn.lrange(queue_name, 0, 100).await.unwrap_or_default();

                for message in messages {
                    if let Ok(task_message) = serde_json::from_str::<Value>(&message) {
                        if let Some(headers) = task_message.get("headers") {
                            if let (Some(task_id), Some(task_name)) = (
                                headers.get("id").and_then(|id| id.as_str()),
                                headers.get("task").and_then(|task| task.as_str()),
                            ) {
                                task_names.insert(task_id.to_string(), task_name.to_string());
                            }
                        }
                    }
                }
            }
        }

        // Get task results from celery-task-meta-* keys
        let task_keys: Vec<String> = conn
            .keys("celery-task-meta-*")
            .await
            .map_err(|e| BrokerError::OperationError(e.to_string()))?;

        for key in task_keys.iter().take(100) {
            // Limit to 100 for performance
            if let Ok(data) = conn.get::<_, String>(key).await {
                if let Ok(task_data) = serde_json::from_str::<Value>(&data) {
                    let task_id = key
                        .strip_prefix("celery-task-meta-")
                        .unwrap_or("unknown")
                        .to_string();

                    // Parse timestamp from task data
                    let timestamp = if let Some(date_done) =
                        task_data.get("date_done").and_then(|d| d.as_str())
                    {
                        date_done
                            .parse::<DateTime<Utc>>()
                            .unwrap_or_else(|_| Utc::now())
                    } else {
                        Utc::now()
                    };

                    // Get task name from queue message if available, otherwise from metadata
                    let task_name = task_names
                        .get(&task_id)
                        .cloned()
                        .or_else(|| {
                            task_data
                                .get("task")
                                .and_then(|t| t.as_str())
                                .map(|s| s.to_string())
                        })
                        .unwrap_or_else(|| "unknown".to_string());

                    let task = Task {
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
                        status: match task_data.get("status").and_then(|s| s.as_str()) {
                            Some("SUCCESS") => TaskStatus::Success,
                            Some("FAILURE") => TaskStatus::Failure,
                            Some("PENDING") => TaskStatus::Pending,
                            Some("RETRY") => TaskStatus::Retry,
                            Some("REVOKED") => TaskStatus::Revoked,
                            Some("STARTED") => TaskStatus::Active,
                            _ => TaskStatus::Pending,
                        },
                        worker: None, // Task metadata doesn't contain worker hostname
                        timestamp,
                        result: task_data.get("result").map(|r| r.to_string()),
                        traceback: task_data
                            .get("traceback")
                            .and_then(|t| t.as_str())
                            .map(|s| s.to_string()),
                    };
                    tasks.push(task);
                }
            }
        }

        // Also add pending tasks from queues that might not have metadata yet
        for queue_name in &queue_names {
            let queue_length: u64 = conn.llen(queue_name).await.unwrap_or(0);
            if queue_length > 0 {
                let messages: Vec<String> =
                    conn.lrange(queue_name, 0, 20).await.unwrap_or_default();

                for message in messages {
                    if let Ok(task_message) = serde_json::from_str::<Value>(&message) {
                        if let Some(headers) = task_message.get("headers") {
                            if let (Some(task_id), Some(task_name)) = (
                                headers.get("id").and_then(|id| id.as_str()),
                                headers.get("task").and_then(|task| task.as_str()),
                            ) {
                                // Only add if not already in our task list
                                if !tasks.iter().any(|t| t.id == task_id) {
                                    // Decode task arguments from body
                                    let (args, kwargs) = if let Some(body) =
                                        task_message.get("body").and_then(|b| b.as_str())
                                    {
                                        if let Ok(decoded) =
                                            base64::engine::general_purpose::STANDARD.decode(body)
                                        {
                                            if let Ok(body_str) = String::from_utf8(decoded) {
                                                if let Ok(body_json) =
                                                    serde_json::from_str::<Value>(&body_str)
                                                {
                                                    let args = body_json
                                                        .get(0)
                                                        .map(|a| a.to_string())
                                                        .unwrap_or_else(|| "[]".to_string());
                                                    let kwargs = body_json
                                                        .get(1)
                                                        .map(|k| k.to_string())
                                                        .unwrap_or_else(|| "{}".to_string());
                                                    (args, kwargs)
                                                } else {
                                                    ("[]".to_string(), "{}".to_string())
                                                }
                                            } else {
                                                ("[]".to_string(), "{}".to_string())
                                            }
                                        } else {
                                            ("[]".to_string(), "{}".to_string())
                                        }
                                    } else {
                                        ("[]".to_string(), "{}".to_string())
                                    };

                                    let task = Task {
                                        id: task_id.to_string(),
                                        name: task_name.to_string(),
                                        args,
                                        kwargs,
                                        status: TaskStatus::Pending,
                                        worker: None,
                                        timestamp: Utc::now(),
                                        result: None,
                                        traceback: None,
                                    };
                                    tasks.push(task);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(tasks)
    }

    async fn parse_queues(&self) -> Result<Vec<Queue>, BrokerError> {
        let mut conn = self.connection.clone();
        let mut queues = Vec::new();
        let mut discovered_queues = std::collections::HashSet::new();

        // First, discover queues from kombu bindings
        let binding_keys: Vec<String> = conn.keys("_kombu.binding.*").await.unwrap_or_default();

        for binding_key in binding_keys {
            if let Some(queue_name) = binding_key.strip_prefix("_kombu.binding.") {
                discovered_queues.insert(queue_name.to_string());
            }
        }

        // Also check for common queue names
        let common_queues = vec!["celery", "default", "priority", "high", "low"];
        for queue_name in common_queues {
            discovered_queues.insert(queue_name.to_string());
        }

        // Check each discovered queue
        for queue_name in discovered_queues {
            let length: u64 = conn.llen(&queue_name).await.unwrap_or(0);

            // Only include queues that exist (have been used) or are standard
            if length > 0 || ["celery", "default"].contains(&queue_name.as_str()) {
                // Estimate consumers from worker data (simplified)
                let consumers = if length > 0 { 1 } else { 0 }; // Simplified consumer count

                queues.push(Queue {
                    name: queue_name,
                    length,
                    consumers,
                });
            }
        }

        // Sort queues by name for consistent display
        queues.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(queues)
    }
}

#[async_trait]
impl Broker for RedisBroker {
    async fn connect(url: &str) -> Result<Self, BrokerError> {
        let client = Client::open(url).map_err(|e| BrokerError::InvalidUrl(e.to_string()))?;

        let connection = client
            .get_multiplexed_tokio_connection()
            .await
            .map_err(|e| BrokerError::ConnectionError(e.to_string()))?;

        Ok(Self {
            _client: client,
            connection,
        })
    }

    async fn get_workers(&self) -> Result<Vec<Worker>, BrokerError> {
        self.parse_workers().await
    }

    async fn get_tasks(&self) -> Result<Vec<Task>, BrokerError> {
        self.parse_tasks().await
    }

    async fn get_queues(&self) -> Result<Vec<Queue>, BrokerError> {
        self.parse_queues().await
    }

    async fn retry_task(&self, task_id: &str) -> Result<(), BrokerError> {
        let mut conn = self.connection.clone();

        // Get the task metadata to extract task information
        let task_key = format!("celery-task-meta-{task_id}");
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

    async fn revoke_task(&self, task_id: &str) -> Result<(), BrokerError> {
        let mut conn = self.connection.clone();

        // Add task to Celery's revoked tasks set
        let revoked_key = "revoked";
        conn.sadd::<_, _, ()>(revoked_key, task_id)
            .await
            .map_err(|e| BrokerError::OperationError(e.to_string()))?;

        // Update task metadata if it exists
        let task_key = format!("celery-task-meta-{task_id}");
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

    async fn purge_queue(&self, queue_name: &str) -> Result<u64, BrokerError> {
        let mut conn = self.connection.clone();

        // Get current queue length for reporting
        let queue_length: u64 = conn
            .llen(queue_name)
            .await
            .map_err(|e| BrokerError::OperationError(e.to_string()))?;

        // Delete all messages from the queue (Redis LIST)
        // Using DEL command to completely remove the list
        let deleted: u64 = conn
            .del(queue_name)
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
