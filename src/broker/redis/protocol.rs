use crate::error::BrokerError;
use crate::models::{Queue, Task, TaskStatus, Worker, WorkerStatus};
use base64::Engine;
use chrono::{DateTime, Utc};
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use serde_json::Value;
use std::collections::HashMap;

// Configuration constants for limits
const MAX_TASK_METADATA_KEYS: usize = 500;
const MAX_TASK_RESULTS: usize = 100;
const MAX_QUEUE_MESSAGES: usize = 100;
const MAX_PENDING_TASKS: usize = 20;
const DEFAULT_WORKER_CONCURRENCY: u32 = 16;

pub struct ProtocolParser;

impl ProtocolParser {
    pub async fn parse_workers(
        connection: &MultiplexedConnection,
    ) -> Result<Vec<Worker>, BrokerError> {
        let mut conn = connection.clone();
        let mut worker_stats: HashMap<String, (u64, u64, Vec<String>)> = HashMap::new();
        let active_workers: HashMap<String, Vec<String>> = HashMap::new();

        // Get task metadata and extract worker information
        Self::get_task_metadata(&mut conn, &mut worker_stats).await?;

        // Extract worker info from queue messages
        Self::extract_worker_info_from_queues(&mut conn, &mut worker_stats).await?;

        // Build the final worker list
        let mut workers = Self::build_worker_list(worker_stats, active_workers);

        // Handle case where no workers are detected
        Self::ensure_default_worker_if_needed(&mut conn, &mut workers).await?;

        Ok(workers)
    }

    async fn get_task_metadata(
        conn: &mut MultiplexedConnection,
        worker_stats: &mut HashMap<String, (u64, u64, Vec<String>)>,
    ) -> Result<(), BrokerError> {
        let task_keys: Vec<String> = conn.keys("celery-task-meta-*").await.map_err(|e| {
            BrokerError::OperationError(format!("Failed to get task metadata keys: {e}"))
        })?;

        for key in task_keys.iter().take(MAX_TASK_METADATA_KEYS) {
            match conn.get::<_, String>(key).await {
                Ok(data) => {
                    match serde_json::from_str::<Value>(&data) {
                        Ok(task_data) => {
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
                        Err(_) => {
                            // Skip malformed task data - log error but continue processing
                            continue;
                        }
                    }
                }
                Err(_) => {
                    // Skip inaccessible keys - continue processing other tasks
                    continue;
                }
            }
        }

        Ok(())
    }

    async fn extract_worker_info_from_queues(
        conn: &mut MultiplexedConnection,
        worker_stats: &mut HashMap<String, (u64, u64, Vec<String>)>,
    ) -> Result<(), BrokerError> {
        let queue_names = vec!["celery", "default", "priority"];

        for queue_name in queue_names {
            match conn.llen::<_, u64>(queue_name).await {
                Ok(queue_length) if queue_length > 0 => {
                    match conn.lrange::<_, Vec<String>>(queue_name, 0, 5).await {
                        Ok(messages) => {
                            for message in &messages {
                                if let Ok(task_message) = serde_json::from_str::<Value>(message) {
                                    if let Some(hostname) =
                                        Self::extract_hostname_from_message(&task_message)
                                    {
                                        let (_processed, _failed, queues) = worker_stats
                                            .entry(hostname)
                                            .or_insert((0, 0, Vec::new()));
                                        if !queues.contains(&queue_name.to_string()) {
                                            queues.push(queue_name.to_string());
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

        Ok(())
    }

    fn extract_hostname_from_message(task_message: &Value) -> Option<String> {
        task_message
            .get("headers")
            .and_then(|headers| headers.get("origin"))
            .and_then(|origin| origin.as_str())
            .map(|origin| {
                // Extract hostname from origin like "gen447152@archflowx13"
                if let Some(at_pos) = origin.find('@') {
                    origin[at_pos + 1..].to_string()
                } else {
                    origin.to_string()
                }
            })
    }

    fn build_worker_list(
        worker_stats: HashMap<String, (u64, u64, Vec<String>)>,
        active_workers: HashMap<String, Vec<String>>,
    ) -> Vec<Worker> {
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
                concurrency: DEFAULT_WORKER_CONCURRENCY,
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

        workers
    }

    async fn ensure_default_worker_if_needed(
        conn: &mut MultiplexedConnection,
        workers: &mut Vec<Worker>,
    ) -> Result<(), BrokerError> {
        if workers.is_empty() {
            let celery_queue_len: u64 = conn.llen("celery").await.unwrap_or(0);
            let task_keys: Vec<String> = conn.keys("celery-task-meta-*").await.map_err(|e| {
                BrokerError::OperationError(format!(
                    "Failed to check for task metadata keys: {e}"
                ))
            })?;
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
                    concurrency: DEFAULT_WORKER_CONCURRENCY,
                    queues: vec!["celery".to_string()],
                    active_tasks: vec![],
                    processed: task_count as u64,
                    failed: 0,
                });
            }
        }

        Ok(())
    }

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

    fn parse_timestamp(task_data: &Value) -> DateTime<Utc> {
        if let Some(date_done) = task_data.get("date_done").and_then(|d| d.as_str()) {
            date_done
                .parse::<DateTime<Utc>>()
                .unwrap_or_else(|_| Utc::now())
        } else {
            Utc::now()
        }
    }

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

    pub async fn parse_queues(
        connection: &MultiplexedConnection,
    ) -> Result<Vec<Queue>, BrokerError> {
        let mut conn = connection.clone();
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
