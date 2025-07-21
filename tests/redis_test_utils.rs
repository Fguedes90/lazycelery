//! Redis Test Utilities
//!
//! This module provides shared utilities for Redis broker testing to eliminate
//! duplication and improve test isolation and maintainability.

use anyhow::Result;
use base64::Engine;
use lazycelery::broker::{redis::RedisBroker, Broker};
use lazycelery::models::{TaskStatus, WorkerStatus};
use redis::{AsyncCommands, Client};
use serde_json::json;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};

// Global counter for database isolation
static DB_COUNTER: AtomicU8 = AtomicU8::new(2);

/// Test database configuration that ensures isolation between tests
#[derive(Clone)]
pub struct TestDatabase {
    pub database_id: u8,
    pub url: String,
    pub client: Option<Client>,
}

impl TestDatabase {
    /// Create a new isolated test database
    pub async fn new() -> Result<Self> {
        let db_id = DB_COUNTER.fetch_add(1, Ordering::SeqCst);
        if db_id > 15 {
            // Redis supports databases 0-15 by default
            return Err(anyhow::anyhow!("Too many concurrent tests"));
        }

        let url = format!("redis://127.0.0.1:6379/{db_id}");

        Ok(TestDatabase {
            database_id: db_id,
            url,
            client: None,
        })
    }

    /// Setup and initialize the test database
    pub async fn setup(&mut self) -> Result<Client> {
        let client = Client::open(self.url.as_str())?;
        let mut conn = client.get_multiplexed_tokio_connection().await?;

        // Clear the database to ensure clean state
        let _: () = redis::cmd("FLUSHDB").query_async(&mut conn).await?;

        self.client = Some(client.clone());
        Ok(client)
    }

    /// Get database client, setting up if needed
    pub async fn client(&mut self) -> Result<Client> {
        if let Some(ref client) = self.client {
            Ok(client.clone())
        } else {
            self.setup().await
        }
    }

    /// Create a broker instance for this test database
    pub async fn broker(&mut self) -> Result<RedisBroker> {
        match RedisBroker::connect(&self.url).await {
            Ok(broker) => Ok(broker),
            Err(_) => Err(anyhow::anyhow!("Redis not available for testing")),
        }
    }

    /// Cleanup the test database
    pub async fn cleanup(&mut self) -> Result<()> {
        if let Some(ref client) = self.client {
            let mut conn = client.get_multiplexed_tokio_connection().await?;
            let _: () = redis::cmd("FLUSHDB").query_async(&mut conn).await?;
        }
        Ok(())
    }
}

/// Builder for creating various types of test data
pub struct TestDataBuilder {
    client: Client,
}

#[allow(dead_code)]
impl TestDataBuilder {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Add basic test task metadata
    pub async fn add_basic_tasks(&self) -> Result<()> {
        let mut conn = self.client.get_multiplexed_tokio_connection().await?;

        // Success task
        let success_task = json!({
            "task": "myapp.tasks.process_data",
            "args": "[1, 2, 3]",
            "kwargs": "{\"timeout\": 30}",
            "status": "SUCCESS",
            "hostname": "worker-1",
            "result": "\"Processing completed\"",
            "traceback": null
        });

        let _: () = conn
            .set("celery-task-meta-basic-success-1", success_task.to_string())
            .await?;

        // Failed task
        let failed_task = json!({
            "task": "myapp.tasks.failing_task",
            "args": "[]",
            "kwargs": "{}",
            "status": "FAILURE",
            "hostname": "worker-2",
            "result": null,
            "traceback": "Traceback (most recent call last):\n  File...\nZeroDivisionError: division by zero"
        });

        let _: () = conn
            .set("celery-task-meta-basic-failure-1", failed_task.to_string())
            .await?;

        Ok(())
    }

    /// Add real Celery protocol formatted data
    pub async fn add_real_celery_data(&self) -> Result<()> {
        let mut conn = self.client.get_multiplexed_tokio_connection().await?;

        // Real Celery task metadata (without hostname - as Celery actually stores)
        let successful_task = json!({
            "status": "SUCCESS",
            "result": 42,
            "traceback": null,
            "children": [],
            "date_done": "2024-01-15T10:30:00.123456+00:00",
            "task_id": "real-success-task"
        });

        let failed_task = json!({
            "status": "FAILURE",
            "result": null,
            "traceback": "Traceback (most recent call last):\n  File \"test.py\", line 1\n    raise ValueError(\"Test error\")\nValueError: Test error",
            "children": [],
            "date_done": "2024-01-15T10:35:00.123456+00:00",
            "task_id": "real-failure-task"
        });

        let pending_task = json!({
            "status": "PENDING",
            "result": null,
            "traceback": null,
            "children": [],
            "task_id": "real-pending-task"
        });

        // Store task metadata
        let _: () = conn
            .set(
                "celery-task-meta-real-success-task",
                successful_task.to_string(),
            )
            .await?;

        let _: () = conn
            .set(
                "celery-task-meta-real-failure-task",
                failed_task.to_string(),
            )
            .await?;

        let _: () = conn
            .set(
                "celery-task-meta-real-pending-task",
                pending_task.to_string(),
            )
            .await?;

        // Add real Celery queue message
        let task_message = json!({
            "body": base64::engine::general_purpose::STANDARD.encode(r#"[[1, 2], {"timeout": 30}, {"callbacks": null, "errbacks": null, "chain": null, "chord": null}]"#),
            "content-encoding": "utf-8",
            "content-type": "application/json",
            "headers": {
                "lang": "py",
                "task": "myapp.tasks.add_numbers",
                "id": "real-queue-task",
                "shadow": null,
                "eta": null,
                "expires": null,
                "group": null,
                "group_index": null,
                "retries": 0,
                "timelimit": [null, null],
                "root_id": "real-queue-task",
                "parent_id": null,
                "argsrepr": "(1, 2)",
                "kwargsrepr": "{\"timeout\": 30}",
                "origin": "gen123@worker-host-1",
                "ignore_result": false,
                "replaced_task_nesting": 0,
                "stamped_headers": null,
                "stamps": {}
            },
            "properties": {
                "correlation_id": "real-queue-task",
                "reply_to": "reply-queue",
                "delivery_mode": 2,
                "delivery_info": {"exchange": "", "routing_key": "celery"},
                "priority": 0,
                "body_encoding": "base64",
                "delivery_tag": "delivery-tag-123"
            }
        });

        // Add to queue
        let _: () = conn.lpush("celery", task_message.to_string()).await?;

        // Add kombu bindings
        let _: () = conn.set("_kombu.binding.celery", "").await?;
        let _: () = conn.set("_kombu.binding.priority", "").await?;

        // Add revoked tasks
        let _: () = conn.sadd("revoked", "revoked-task-1").await?;

        Ok(())
    }

    /// Add queue test data
    pub async fn add_queue_data(&self) -> Result<()> {
        let mut conn = self.client.get_multiplexed_tokio_connection().await?;

        // Add items to queues
        let _: () = conn.lpush("celery", "task1").await?;
        let _: () = conn.lpush("celery", "task2").await?;
        let _: () = conn.lpush("priority", "urgent_task").await?;

        Ok(())
    }

    /// Add performance test data (many tasks)
    pub async fn add_performance_data(&self, count: usize) -> Result<()> {
        let mut conn = self.client.get_multiplexed_tokio_connection().await?;

        for i in 0..count {
            let task_data = json!({
                "status": match i % 3 {
                    0 => "SUCCESS",
                    1 => "FAILURE",
                    _ => "PENDING"
                },
                "result": i,
                "task_id": format!("perf-task-{}", i)
            });

            let _: () = conn
                .set(
                    format!("celery-task-meta-perf-task-{i}"),
                    task_data.to_string(),
                )
                .await?;
        }

        Ok(())
    }

    /// Add malformed data for edge case testing
    pub async fn add_malformed_data(&self) -> Result<()> {
        let mut conn = self.client.get_multiplexed_tokio_connection().await?;

        // Various malformed data scenarios
        let _: () = conn
            .set("celery-task-meta-malformed", "invalid json")
            .await?;
        let _: () = conn.set("celery-task-meta-empty", "").await?;
        let _: () = conn.set("celery-task-meta-null", "null").await?;

        // Incomplete task (missing fields)
        let incomplete_task = json!({
            "status": "SUCCESS"
            // Missing other fields
        });
        let _: () = conn
            .set("celery-task-meta-incomplete", incomplete_task.to_string())
            .await?;

        Ok(())
    }

    /// Add specific task for retry testing
    pub async fn add_retry_test_task(&self, task_id: &str) -> Result<()> {
        let mut conn = self.client.get_multiplexed_tokio_connection().await?;

        let failed_task = json!({
            "status": "FAILURE",
            "result": null,
            "traceback": "Test error for retry",
            "task_id": task_id,
            "retries": 0
        });

        let _: () = conn
            .set(
                format!("celery-task-meta-{task_id}"),
                failed_task.to_string(),
            )
            .await?;

        Ok(())
    }

    /// Add custom task with specific properties
    pub async fn add_custom_task(
        &self,
        task_id: &str,
        properties: HashMap<&str, serde_json::Value>,
    ) -> Result<()> {
        let mut conn = self.client.get_multiplexed_tokio_connection().await?;

        let mut task_data = json!({
            "task": "custom.task",
            "status": "PENDING",
            "args": "[]",
            "kwargs": "{}",
            "task_id": task_id
        });

        // Merge custom properties
        for (key, value) in properties {
            task_data[key] = value;
        }

        let _: () = conn
            .set(format!("celery-task-meta-{task_id}"), task_data.to_string())
            .await?;

        Ok(())
    }
}

/// Assertion helpers for test validation
pub struct TestAssertions;

#[allow(dead_code)]
impl TestAssertions {
    /// Assert that a task has expected properties
    pub fn assert_task_properties(
        tasks: &[lazycelery::models::Task],
        task_id: &str,
        expected_status: TaskStatus,
        should_have_result: bool,
        should_have_traceback: bool,
    ) {
        let task = tasks
            .iter()
            .find(|t| t.id == task_id)
            .unwrap_or_else(|| panic!("Task {} not found in {} tasks", task_id, tasks.len()));

        assert_eq!(
            task.status, expected_status,
            "Task {} should have status {:?}, but has {:?}",
            task_id, expected_status, task.status
        );

        if should_have_result {
            assert!(task.result.is_some(), "Task {task_id} should have a result");
        } else {
            assert!(
                task.result.is_none() || task.result.as_ref().unwrap().is_empty(),
                "Task {task_id} should not have a result"
            );
        }

        if should_have_traceback {
            assert!(
                task.traceback.is_some(),
                "Task {task_id} should have a traceback"
            );
        }
    }

    /// Assert worker properties
    pub fn assert_worker_properties(
        workers: &[lazycelery::models::Worker],
        min_count: usize,
        should_have_activity: bool,
    ) {
        assert!(
            workers.len() >= min_count,
            "Expected at least {} workers, found {}",
            min_count,
            workers.len()
        );

        if !workers.is_empty() {
            let worker = &workers[0];
            assert!(
                !worker.hostname.is_empty(),
                "Worker should have non-empty hostname"
            );
            assert!(
                matches!(worker.status, WorkerStatus::Online | WorkerStatus::Offline),
                "Worker should have valid status, got {:?}",
                worker.status
            );
            assert!(
                worker.concurrency > 0,
                "Worker should have positive concurrency"
            );

            if should_have_activity {
                assert!(
                    worker.processed > 0 || worker.failed > 0,
                    "Worker should show some activity (processed: {}, failed: {})",
                    worker.processed,
                    worker.failed
                );
            }
        }
    }

    /// Assert queue properties
    pub fn assert_queue_properties(
        queues: &[lazycelery::models::Queue],
        min_count: usize,
        expected_queue: Option<(&str, usize)>,
    ) {
        assert!(
            queues.len() >= min_count,
            "Expected at least {} queues, found {}",
            min_count,
            queues.len()
        );

        if let Some((queue_name, min_length)) = expected_queue {
            let queue = queues
                .iter()
                .find(|q| q.name == queue_name)
                .unwrap_or_else(|| panic!("Queue {queue_name} not found"));

            assert!(
                queue.length >= min_length as u64,
                "Queue {} should have at least {} items, has {}",
                queue_name,
                min_length,
                queue.length
            );
        }
    }

    /// Assert Redis operation was successful
    pub async fn assert_redis_key_exists(
        client: &Client,
        key: &str,
        should_exist: bool,
    ) -> Result<()> {
        let mut conn = client.get_multiplexed_tokio_connection().await?;
        let exists: bool = conn.exists(key).await?;

        if should_exist {
            assert!(exists, "Redis key {key} should exist");
        } else {
            assert!(!exists, "Redis key {key} should not exist");
        }

        Ok(())
    }

    /// Assert task is in revoked set
    pub async fn assert_task_revoked(
        client: &Client,
        task_id: &str,
        should_be_revoked: bool,
    ) -> Result<()> {
        let mut conn = client.get_multiplexed_tokio_connection().await?;
        let is_revoked: bool = conn.sismember("revoked", task_id).await?;

        if should_be_revoked {
            assert!(is_revoked, "Task {task_id} should be in revoked set");
        } else {
            assert!(!is_revoked, "Task {task_id} should not be in revoked set");
        }

        Ok(())
    }

    /// Assert task metadata was updated
    pub async fn assert_task_metadata_updated(
        client: &Client,
        task_id: &str,
        expected_status: &str,
        should_have_retries: bool,
    ) -> Result<()> {
        let mut conn = client.get_multiplexed_tokio_connection().await?;
        let key = format!("celery-task-meta-{task_id}");

        let data: String = conn.get(&key).await?;
        let task_json: serde_json::Value = serde_json::from_str(&data)?;

        assert_eq!(
            task_json["status"], expected_status,
            "Task {} should have status {}, got {}",
            task_id, expected_status, task_json["status"]
        );

        if should_have_retries {
            let retries = task_json["retries"].as_i64().unwrap_or(0);
            assert!(retries > 0, "Task {task_id} should have retries > 0");
        }

        Ok(())
    }
}

/// Skip test if Redis is not available
pub fn skip_if_redis_unavailable(result: Result<()>) -> Result<()> {
    match result {
        Err(e) if e.to_string().contains("Redis not available") => {
            eprintln!("Skipping integration test: Redis not available");
            Ok(())
        }
        other => other,
    }
}

/// Create test database with automatic cleanup
pub async fn with_test_db<F, Fut, T>(test_fn: F) -> Result<T>
where
    F: FnOnce(TestDatabase) -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut db = TestDatabase::new().await?;
    let _client = db
        .setup()
        .await
        .map_err(|_| anyhow::anyhow!("Redis not available"))?;

    let result = test_fn(db.clone()).await;
    let _ = db.cleanup().await; // Best effort cleanup

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_isolation() -> Result<()> {
        let mut db1 = TestDatabase::new().await?;
        let mut db2 = TestDatabase::new().await?;

        // Ensure different databases
        assert_ne!(db1.database_id, db2.database_id);
        assert_ne!(db1.url, db2.url);

        // Test that setup works
        if db1.setup().await.is_ok() && db2.setup().await.is_ok() {
            let client1 = db1.client().await?;
            let client2 = db2.client().await?;

            let builder1 = TestDataBuilder::new(client1);
            let builder2 = TestDataBuilder::new(client2);

            // Add different data to each database
            builder1.add_basic_tasks().await?;
            builder2.add_real_celery_data().await?;

            // Verify isolation (each database should only have its own data)
            let broker1 = db1.broker().await?;
            let broker2 = db2.broker().await?;

            let tasks1 = broker1.get_tasks().await?;
            let tasks2 = broker2.get_tasks().await?;

            // Each should have different tasks
            let has_basic = tasks1.iter().any(|t| t.id.starts_with("basic-"));
            let has_real = tasks2.iter().any(|t| t.id.starts_with("real-"));

            if has_basic && has_real {
                // Verify they don't cross-contaminate
                assert!(!tasks1.iter().any(|t| t.id.starts_with("real-")));
                assert!(!tasks2.iter().any(|t| t.id.starts_with("basic-")));
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_test_data_builder() -> Result<()> {
        skip_if_redis_unavailable(
            async {
                with_test_db(|mut db| async move {
                    let client = db.client().await?;
                    let builder = TestDataBuilder::new(client.clone());

                    // Test basic data creation
                    builder.add_basic_tasks().await?;

                    let broker = db.broker().await?;
                    let tasks = broker.get_tasks().await?;

                    // Should find the basic tasks
                    assert!(tasks.iter().any(|t| t.id == "basic-success-1"));
                    assert!(tasks.iter().any(|t| t.id == "basic-failure-1"));

                    Ok(())
                })
                .await
            }
            .await,
        )
    }
}
