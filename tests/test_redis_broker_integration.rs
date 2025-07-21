//! Comprehensive Redis Broker Integration Tests
//!
//! This module contains comprehensive integration tests using real Celery protocol format.
//! These tests validate the broker's ability to parse authentic Celery messages, handle
//! task operations (retry, revoke), and perform well with realistic data volumes.
//! For basic connection and parsing logic tests, see test_redis_broker_basic.rs.

mod redis_test_utils;

use anyhow::Result;
use base64::Engine;
use lazycelery::broker::Broker;
use lazycelery::models::TaskStatus;
use redis::AsyncCommands;
use redis_test_utils::*;
use serde_json::json;
use std::time::Duration;

#[tokio::test]
async fn test_real_celery_worker_discovery() -> Result<()> {
    skip_if_redis_unavailable(
        async {
            with_test_db(|mut db| async move {
                let client = db.client().await?;
                let builder = TestDataBuilder::new(client.clone());
                builder.add_real_celery_data().await?;

                let broker = db.broker().await?;
                let workers = broker.get_workers().await?;

                // Should discover workers based on task activity
                TestAssertions::assert_worker_properties(&workers, 1, false);

                if !workers.is_empty() {
                    let worker = &workers[0];
                    assert!(!worker.queues.is_empty(), "Worker should have queues");
                }

                Ok(())
            })
            .await
        }
        .await,
    )
}

#[tokio::test]
async fn test_real_celery_task_parsing() -> Result<()> {
    skip_if_redis_unavailable(
        async {
            with_test_db(|mut db| async move {
                let client = db.client().await?;
                let builder = TestDataBuilder::new(client.clone());
                builder.add_real_celery_data().await?;

                let broker = db.broker().await?;
                let tasks = broker.get_tasks().await?;

                // Should find tasks from metadata + queue
                assert!(tasks.len() >= 3, "Should find at least 3 tasks");

                // Verify successful task (isolated test, should be clean)
                let success_task = tasks
                    .iter()
                    .find(|t| t.id == "real-success-task")
                    .expect("Should find successful task");

                assert_eq!(success_task.status, TaskStatus::Success);
                assert_eq!(success_task.result, Some("42".to_string()));

                // Verify failed task
                let failed_task = tasks
                    .iter()
                    .find(|t| t.id == "real-failure-task")
                    .expect("Should find failed task");

                assert_eq!(failed_task.status, TaskStatus::Failure);
                assert!(failed_task.traceback.is_some());
                assert!(failed_task
                    .traceback
                    .as_ref()
                    .unwrap()
                    .contains("ValueError"));

                // Verify pending task from queue
                let pending_task = tasks.iter().find(|t| t.name == "myapp.tasks.add_numbers");
                if let Some(task) = pending_task {
                    assert_eq!(task.status, TaskStatus::Pending);
                    assert!(!task.args.is_empty());
                    assert!(!task.kwargs.is_empty());
                }

                Ok(())
            })
            .await
        }
        .await,
    )
}

#[tokio::test]
async fn test_real_celery_queue_discovery() -> Result<()> {
    skip_if_redis_unavailable(
        async {
            with_test_db(|mut db| async move {
                let client = db.client().await?;
                let builder = TestDataBuilder::new(client.clone());
                builder.add_real_celery_data().await?;

                let broker = db.broker().await?;
                let queues = broker.get_queues().await?;

                // Should discover queues based on bindings + activity
                TestAssertions::assert_queue_properties(
                    &queues,
                    1,                   // min_count
                    Some(("celery", 1)), // expected celery queue with 1 item
                );

                Ok(())
            })
            .await
        }
        .await,
    )
}

#[tokio::test]
async fn test_task_retry_functionality() -> Result<()> {
    skip_if_redis_unavailable(
        async {
            with_test_db(|mut db| async move {
                let client = db.client().await?;
                let builder = TestDataBuilder::new(client.clone());
                builder.add_real_celery_data().await?;

                let broker = db.broker().await?;
                let failed_task_id = "real-failure-task";
                let success_task_id = "real-success-task";

                // Test retry of failed task
                let result = broker.retry_task(failed_task_id).await;
                assert!(result.is_ok(), "Should successfully retry failed task");

                // Verify status was updated
                TestAssertions::assert_task_metadata_updated(
                    &client,
                    failed_task_id,
                    "RETRY",
                    true, // should have retries
                )
                .await?;

                // Test retry of successful task (should fail)
                let result = broker.retry_task(success_task_id).await;
                assert!(result.is_err(), "Should fail to retry successful task");

                // Test retry of nonexistent task (should fail)
                let result = broker.retry_task("nonexistent-task").await;
                assert!(result.is_err(), "Should fail to retry nonexistent task");

                Ok(())
            })
            .await
        }
        .await,
    )
}

#[tokio::test]
async fn test_task_revoke_functionality() -> Result<()> {
    skip_if_redis_unavailable(
        async {
            with_test_db(|mut db| async move {
                let client = db.client().await?;
                let builder = TestDataBuilder::new(client.clone());
                builder.add_real_celery_data().await?;

                let broker = db.broker().await?;
                let task_id = "real-success-task";

                // Revoke a task
                let result = broker.revoke_task(task_id).await;
                assert!(result.is_ok(), "Should successfully revoke task");

                // Verify task was added to revoked set
                TestAssertions::assert_task_revoked(&client, task_id, true).await?;

                // Verify metadata was updated (if exists)
                let mut conn = client.get_multiplexed_tokio_connection().await?;
                if let Ok(updated_data) = conn
                    .get::<_, String>(&format!("celery-task-meta-{task_id}"))
                    .await
                {
                    let task_json: serde_json::Value = serde_json::from_str(&updated_data)?;
                    assert_eq!(task_json["status"], "REVOKED");
                }

                Ok(())
            })
            .await
        }
        .await,
    )
}

#[tokio::test]
async fn test_task_timestamp_parsing() -> Result<()> {
    skip_if_redis_unavailable(
        async {
            with_test_db(|mut db| async move {
                let client = db.client().await?;
                let builder = TestDataBuilder::new(client.clone());
                builder.add_real_celery_data().await?;

                let broker = db.broker().await?;
                let tasks = broker.get_tasks().await?;

                // Verify correct timestamp parsing
                let success_task = tasks
                    .iter()
                    .find(|t| t.id == "real-success-task")
                    .expect("Should find successful task");

                // Verify timestamp was parsed correctly
                assert!(
                    success_task.timestamp.timestamp() > 0,
                    "Should have valid timestamp"
                );

                Ok(())
            })
            .await
        }
        .await,
    )
}

#[tokio::test]
async fn test_base64_task_body_decoding() -> Result<()> {
    skip_if_redis_unavailable(
        async {
            with_test_db(|mut db| async move {
                let client = db.client().await?;
                let mut conn = client.get_multiplexed_tokio_connection().await?;

                // Add message with base64 encoded body
                let task_args = json!([[10, 20], {"multiply": true}]);
                let encoded_body =
                    base64::engine::general_purpose::STANDARD.encode(task_args.to_string());

                let task_message = json!({
                    "body": encoded_body,
                    "headers": {
                        "task": "math.multiply",
                        "id": "base64-test-task"
                    }
                });

                let _: () = conn.lpush("celery", task_message.to_string()).await?;

                let broker = db.broker().await?;
                let tasks = broker.get_tasks().await?;

                // Should find task with decoded args
                let decoded_task = tasks
                    .iter()
                    .find(|t| t.name == "math.multiply")
                    .expect("Should find task with decoded body");

                assert!(decoded_task.args.contains("10"));
                assert!(decoded_task.args.contains("20"));
                assert!(decoded_task.kwargs.contains("multiply"));

                Ok(())
            })
            .await
        }
        .await,
    )
}

#[tokio::test]
async fn test_performance_with_large_dataset() -> Result<()> {
    skip_if_redis_unavailable(
        async {
            with_test_db(|mut db| async move {
                let client = db.client().await?;
                let builder = TestDataBuilder::new(client.clone());

                // Add many tasks for performance testing
                builder.add_performance_data(50).await?;

                let broker = db.broker().await?;

                // Measure execution time
                let start = std::time::Instant::now();
                let tasks = broker.get_tasks().await?;
                let duration = start.elapsed();

                // Verify results
                assert!(tasks.len() >= 50, "Should find all tasks");
                assert!(
                    duration < Duration::from_millis(5000),
                    "Should complete within 5 seconds"
                );

                // Verify worker discovery based on activity
                let workers = broker.get_workers().await?;

                // With 50 processed tasks, should detect activity
                if !workers.is_empty() {
                    let worker = &workers[0];
                    // With SUCCESS and FAILURE tasks, should show activity
                    assert!(
                    worker.processed >= 16 || worker.failed >= 16, // 50/3 = ~16 of each type
                    "Worker should show activity from processed tasks (processed: {}, failed: {})",
                    worker.processed,
                    worker.failed
                );
                }

                Ok(())
            })
            .await
        }
        .await,
    )
}

#[tokio::test]
async fn test_edge_cases_and_malformed_data() -> Result<()> {
    skip_if_redis_unavailable(
        async {
            with_test_db(|mut db| async move {
                let client = db.client().await?;
                let builder = TestDataBuilder::new(client.clone());

                // Add malformed data
                builder.add_malformed_data().await?;

                let broker = db.broker().await?;

                // Should handle malformed data gracefully
                let result = broker.get_tasks().await;
                assert!(result.is_ok(), "Should handle malformed data gracefully");

                let tasks = result.unwrap();
                // Should find at least the incomplete task
                let incomplete = tasks.iter().find(|t| t.id == "incomplete");
                if let Some(task) = incomplete {
                    assert_eq!(task.status, TaskStatus::Success);
                    assert_eq!(task.args, "[]"); // Default value
                    assert_eq!(task.kwargs, "{}"); // Default value
                }

                Ok(())
            })
            .await
        }
        .await,
    )
}
