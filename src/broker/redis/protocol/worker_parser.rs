//! Worker parser for Redis Celery protocol
//!
//! This module handles parsing worker information from Redis data structures.
//! It extracts worker statistics, status, and queue assignments from task metadata
//! and queue messages.

use crate::error::BrokerError;
use crate::models::{Worker, WorkerStatus};
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use serde_json::Value;
use std::collections::HashMap;

// Configuration constants for worker parsing
const MAX_TASK_METADATA_KEYS: usize = 500;
const DEFAULT_WORKER_CONCURRENCY: u32 = 16;

/// Parser for worker-related data from Redis
pub struct WorkerParser;

impl WorkerParser {
    /// Parse workers from Redis connection
    ///
    /// Extracts worker information from task metadata and queue messages to build
    /// a comprehensive view of active workers, their status, and statistics.
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

    /// Extract worker statistics from task metadata
    ///
    /// Processes completed task metadata to extract worker performance statistics
    /// including processed and failed task counts.
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

    /// Extract worker information from queue messages
    ///
    /// Analyzes pending tasks in queues to identify worker hostnames and
    /// associated queue assignments.
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

    /// Extract hostname from a task message
    ///
    /// Parses the 'origin' field from task headers to extract the worker hostname.
    /// Handles various origin formats like "gen447152@archflowx13".
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

    /// Build the final worker list from collected statistics
    ///
    /// Converts raw worker statistics into Worker structs with appropriate
    /// status determination and queue assignments.
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

    /// Ensure at least one worker exists if activity is detected
    ///
    /// Creates a default worker when no specific workers are found but
    /// there is evidence of Celery activity (pending tasks or completed tasks).
    async fn ensure_default_worker_if_needed(
        conn: &mut MultiplexedConnection,
        workers: &mut Vec<Worker>,
    ) -> Result<(), BrokerError> {
        if workers.is_empty() {
            let celery_queue_len: u64 = conn.llen("celery").await.unwrap_or(0);
            let task_keys: Vec<String> = conn.keys("celery-task-meta-*").await.map_err(|e| {
                BrokerError::OperationError(format!("Failed to check for task metadata keys: {e}"))
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
}
