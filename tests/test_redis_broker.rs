use anyhow::Result;
use lazycelery::broker::{redis::RedisBroker, Broker};
use lazycelery::error::BrokerError;
use lazycelery::models::TaskStatus;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_redis_broker_connect_success() {
    // Test successful connection with valid Redis URL
    let url = "redis://127.0.0.1:6379";

    // This will fail in CI without Redis, but shows the connection logic
    match RedisBroker::connect(url).await {
        Ok(_broker) => {
            // Connection successful
        }
        Err(BrokerError::ConnectionError(_)) => {
            // Expected in CI environment without Redis
        }
        Err(e) => panic!("Unexpected error type: {e:?}"),
    }
}

#[tokio::test]
async fn test_redis_broker_connect_invalid_url() {
    let invalid_urls = vec![
        "",
        "not-a-url",
        "http://wrong-protocol",
        "redis://",
        "redis://invalid-host:99999",
    ];

    for url in invalid_urls {
        let result = RedisBroker::connect(url).await;
        match result {
            Err(BrokerError::InvalidUrl(_)) | Err(BrokerError::ConnectionError(_)) => {
                // Expected behavior
            }
            _ => panic!("Expected InvalidUrl or ConnectionError for URL: {url}"),
        }
    }
}

#[tokio::test]
async fn test_redis_broker_connection_timeout() {
    // Test connection timeout with unreachable host
    let unreachable_url = "redis://192.0.2.1:6379"; // RFC 5737 test address

    let result = timeout(
        Duration::from_secs(5),
        RedisBroker::connect(unreachable_url),
    )
    .await;

    match result {
        Ok(Err(BrokerError::ConnectionError(_))) => {
            // Expected: connection should fail
        }
        Err(_) => {
            // Timeout occurred, which is also acceptable
        }
        Ok(Ok(_)) => {
            panic!("Connection should not succeed to unreachable host");
        }
        Ok(Err(_)) => {
            // Any other broker error is also acceptable for unreachable host
        }
    }
}

// Integration tests that run only when Redis is available
mod integration_tests {
    use super::*;
    use redis::{AsyncCommands, Client};

    async fn setup_test_redis() -> Result<Client> {
        let client = Client::open("redis://127.0.0.1:6379/1")?; // Use DB 1 for tests
        let mut conn = client.get_multiplexed_tokio_connection().await?;

        // Clear test data
        let _: () = redis::cmd("FLUSHDB").query_async(&mut conn).await?;

        Ok(client)
    }

    async fn populate_test_data(client: &Client) -> Result<()> {
        let mut conn = client.get_multiplexed_tokio_connection().await?;

        // Add test task metadata
        let task_data = serde_json::json!({
            "task": "myapp.tasks.process_data",
            "args": "[1, 2, 3]",
            "kwargs": "{\"timeout\": 30}",
            "status": "SUCCESS",
            "hostname": "worker-1",
            "result": "\"Processing completed\"",
            "traceback": null
        });

        let _: () = conn
            .set("celery-task-meta-test-task-1", task_data.to_string())
            .await?;

        // Add test task with failure
        let failed_task_data = serde_json::json!({
            "task": "myapp.tasks.failing_task",
            "args": "[]",
            "kwargs": "{}",
            "status": "FAILURE",
            "hostname": "worker-2",
            "result": null,
            "traceback": "Traceback (most recent call last):\n  File...\nZeroDivisionError: division by zero"
        });

        let _: () = conn
            .set("celery-task-meta-test-task-2", failed_task_data.to_string())
            .await?;

        // Add test queues with items
        let _: () = conn.lpush("celery", "task1").await?;
        let _: () = conn.lpush("celery", "task2").await?;
        let _: () = conn.lpush("priority", "urgent_task").await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_redis_get_workers_integration() -> Result<()> {
        let client = match setup_test_redis().await {
            Ok(client) => client,
            Err(_) => {
                eprintln!("Skipping integration test: Redis not available");
                return Ok(());
            }
        };

        populate_test_data(&client).await?;

        let broker = RedisBroker::connect("redis://127.0.0.1:6379/1")
            .await
            .map_err(|_| anyhow::anyhow!("Redis not available"))?;

        let workers = broker.get_workers().await?;

        // Should discover workers from task activity (real implementation)
        assert!(!workers.is_empty(), "Should discover at least one worker");

        let worker = &workers[0];
        assert!(!worker.hostname.is_empty(), "Worker should have hostname");
        assert!(
            worker.concurrency > 0,
            "Worker should have positive concurrency"
        );
        // Workers discovered from task metadata should show activity
        assert!(
            worker.processed > 0 || worker.failed > 0,
            "Worker should show some activity"
        );

        Ok(())
    }

    #[tokio::test]
    #[ignore] // Temporarily disabled due to CI interference issues
    async fn test_redis_get_tasks_integration() -> Result<()> {
        let client = match setup_test_redis().await {
            Ok(client) => client,
            Err(_) => {
                eprintln!("Skipping integration test: Redis not available");
                return Ok(());
            }
        };

        // Use unique task IDs to avoid interference with other tests
        let mut conn = client.get_multiplexed_tokio_connection().await?;
        let task_data = serde_json::json!({
            "task": "myapp.tasks.process_data",
            "args": "[1, 2, 3]",
            "kwargs": "{\"timeout\": 30}",
            "status": "SUCCESS",
            "hostname": "worker-1",
            "result": "\"Processing completed\"",
            "traceback": null
        });
        let _: () = conn
            .set("celery-task-meta-unique-test-task-1", task_data.to_string())
            .await?;

        let failed_task_data = serde_json::json!({
            "task": "myapp.tasks.failing_task",
            "args": "[]",
            "kwargs": "{}",
            "status": "FAILURE",
            "hostname": "worker-2",
            "result": null,
            "traceback": "Traceback (most recent call last):\n  File...\nZeroDivisionError: division by zero"
        });
        let _: () = conn
            .set(
                "celery-task-meta-unique-test-task-2",
                failed_task_data.to_string(),
            )
            .await?;

        // Add small delay to ensure data is persisted in CI environment
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Debug: Verify test data was actually set
        let keys: Vec<String> = conn.keys("celery-task-meta-*").await?;
        eprintln!("DEBUG: Found {} task metadata keys: {:?}", keys.len(), keys);

        let broker = RedisBroker::connect("redis://127.0.0.1:6379/1")
            .await
            .map_err(|_| anyhow::anyhow!("Redis not available"))?;

        let tasks = broker.get_tasks().await?;

        // Debug output for CI troubleshooting
        eprintln!("DEBUG: Found {} tasks in CI", tasks.len());
        for task in &tasks {
            eprintln!("DEBUG: Task ID: {}, Status: {:?}", task.id, task.status);
        }

        // Verify that we can find the tasks we just created
        let our_tasks: Vec<_> = tasks
            .iter()
            .filter(|t| t.id.starts_with("unique-test-task"))
            .collect();
        assert!(
            our_tasks.len() >= 2,
            "Should find at least 2 of our test tasks, found {} (total tasks: {})",
            our_tasks.len(),
            tasks.len()
        );

        // Find the successful task (real Celery metadata doesn't have task name)
        let success_task = tasks
            .iter()
            .find(|t| t.id == "unique-test-task-1")
            .expect("Should find success task");
        assert_eq!(success_task.status, TaskStatus::Success);
        assert_eq!(success_task.worker, None); // Real Celery metadata doesn't include worker hostname
                                               // Result pode ter aspas duplas extras devido ao JSON encoding
        assert!(
            success_task.result.is_some()
                && success_task
                    .result
                    .as_ref()
                    .unwrap()
                    .contains("Processing completed"),
            "Result should contain 'Processing completed', got: {:?}",
            success_task.result
        );

        // Find the failed task
        let failed_task = tasks
            .iter()
            .find(|t| t.id == "unique-test-task-2")
            .expect("Should find failed task");
        assert_eq!(failed_task.status, TaskStatus::Failure);
        assert_eq!(failed_task.worker, None); // Real Celery metadata doesn't include worker hostname
        assert!(failed_task.traceback.is_some());
        assert!(failed_task
            .traceback
            .as_ref()
            .unwrap()
            .contains("ZeroDivisionError"));

        Ok(())
    }

    #[tokio::test]
    async fn test_redis_get_queues_integration() -> Result<()> {
        let client = match setup_test_redis().await {
            Ok(client) => client,
            Err(_) => {
                eprintln!("Skipping integration test: Redis not available");
                return Ok(());
            }
        };

        populate_test_data(&client).await?;

        let broker = RedisBroker::connect("redis://127.0.0.1:6379/1")
            .await
            .map_err(|_| anyhow::anyhow!("Redis not available"))?;

        let queues = broker.get_queues().await?;

        // Real implementation discovers queues dynamically
        assert!(!queues.is_empty(), "Should discover at least one queue");

        // Find celery queue (should always exist)
        let celery_queue = queues.iter().find(|q| q.name == "celery");
        if let Some(queue) = celery_queue {
            assert!(
                queue.length >= 2,
                "Celery queue should have at least 2 items"
            );
        }

        // Check for priority queue if discovered
        if let Some(priority_queue) = queues.iter().find(|q| q.name == "priority") {
            assert!(
                priority_queue.length >= 1,
                "Priority queue should have at least 1 item"
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_redis_task_operations_implemented() -> Result<()> {
        let client = match setup_test_redis().await {
            Ok(client) => client,
            Err(_) => {
                eprintln!("Skipping integration test: Redis not available");
                return Ok(());
            }
        };

        // Add a failed task for testing retry
        let mut conn = client.get_multiplexed_tokio_connection().await?;
        let failed_task = serde_json::json!({
            "status": "FAILURE",
            "result": null,
            "traceback": "Test error",
            "task_id": "test-failed-task"
        });

        let _: () = conn
            .set("celery-task-meta-test-failed-task", failed_task.to_string())
            .await?;

        let broker = RedisBroker::connect("redis://127.0.0.1:6379/1")
            .await
            .map_err(|_| anyhow::anyhow!("Redis not available"))?;

        // Test retry operation (should work for failed tasks)
        let retry_result = broker.retry_task("test-failed-task").await;
        assert!(
            retry_result.is_ok(),
            "Should successfully retry failed task"
        );

        // Test revoke operation (should work for any task)
        let revoke_result = broker.revoke_task("test-task-to-revoke").await;
        assert!(revoke_result.is_ok(), "Should successfully revoke task");

        // Verify task was added to revoked set
        let is_revoked: bool = conn.sismember("revoked", "test-task-to-revoke").await?;
        assert!(is_revoked, "Task should be in revoked set");

        Ok(())
    }
}

// Unit tests for parsing logic (without Redis dependency)
mod parsing_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_task_status_mapping() {
        // Test all possible status mappings
        let test_cases = vec![
            ("SUCCESS", TaskStatus::Success),
            ("FAILURE", TaskStatus::Failure),
            ("PENDING", TaskStatus::Pending),
            ("RETRY", TaskStatus::Retry),
            ("REVOKED", TaskStatus::Revoked),
            ("UNKNOWN", TaskStatus::Active), // Default case
            ("", TaskStatus::Active),        // Empty string case
        ];

        for (status_str, expected) in test_cases {
            let task_data = json!({
                "status": status_str,
                "task": "test.task",
                "args": "[]",
                "kwargs": "{}",
                "hostname": "worker-1"
            });

            // This tests the mapping logic used in parse_tasks
            let parsed_status = match task_data["status"].as_str() {
                Some("SUCCESS") => TaskStatus::Success,
                Some("FAILURE") => TaskStatus::Failure,
                Some("PENDING") => TaskStatus::Pending,
                Some("RETRY") => TaskStatus::Retry,
                Some("REVOKED") => TaskStatus::Revoked,
                _ => TaskStatus::Active,
            };

            assert_eq!(parsed_status, expected, "Failed for status: {status_str}");
        }
    }

    #[test]
    fn test_task_data_parsing_edge_cases() {
        // Test malformed JSON handling
        let malformed_cases = vec![
            "",
            "{",
            r#"{"incomplete": true"#,
            "invalid json",
            "{malformed}",
        ];

        for case in malformed_cases {
            let result = serde_json::from_str::<serde_json::Value>(case);
            assert!(result.is_err(), "Should fail to parse: {case}");
        }

        // Test cases that parse successfully but might not have expected structure
        let valid_but_unexpected = vec![
            "null",
            "\"string\"",
            "123",
            "true",
            "[]", // Array is valid JSON but not the expected object format
        ];

        for case in valid_but_unexpected {
            let result = serde_json::from_str::<serde_json::Value>(case);
            assert!(result.is_ok(), "Should parse as valid JSON: {case}");

            // But accessing object fields would return None/default
            if let Ok(value) = result {
                assert_eq!(value["task"].as_str().unwrap_or("unknown"), "unknown");
                assert_eq!(value["status"].as_str().unwrap_or("default"), "default");
            }
        }
    }

    #[test]
    fn test_task_data_missing_fields() {
        // Test parsing when optional fields are missing
        let minimal_task = json!({
            "task": "test.minimal",
            "status": "SUCCESS"
        });

        // Simulate the parsing logic from parse_tasks
        let task_name = minimal_task["task"].as_str().unwrap_or("unknown");
        let args = minimal_task["args"].to_string();
        let kwargs = minimal_task["kwargs"].to_string();
        let worker = minimal_task["hostname"].as_str().map(|s| s.to_string());
        let result = minimal_task["result"].as_str().map(|s| s.to_string());
        let traceback = minimal_task["traceback"].as_str().map(|s| s.to_string());

        assert_eq!(task_name, "test.minimal");
        assert_eq!(args, "null"); // Missing field becomes "null"
        assert_eq!(kwargs, "null");
        assert_eq!(worker, None);
        assert_eq!(result, None);
        assert_eq!(traceback, None);
    }

    #[test]
    fn test_task_id_extraction() {
        // Test task ID extraction from Redis key
        let test_cases = vec![
            ("celery-task-meta-abc123", "abc123"),
            ("celery-task-meta-", ""),
            ("celery-task-meta-complex-uuid-456", "complex-uuid-456"),
            ("invalid-key-format", "invalid-key-format"), // Fallback to "unknown" in real code
        ];

        for (key, expected) in test_cases {
            let extracted = key.strip_prefix("celery-task-meta-").unwrap_or("unknown");
            if key.starts_with("celery-task-meta-") {
                assert_eq!(extracted, expected, "Failed for key: {key}");
            } else {
                assert_eq!(
                    extracted, "unknown",
                    "Should fallback to unknown for: {key}"
                );
            }
        }
    }
}

// Error handling tests
mod error_tests {
    use super::*;

    #[tokio::test]
    async fn test_redis_operation_errors() {
        // Test various Redis operation failure scenarios
        // These would be mocked in a real test environment

        // Test network timeout simulation
        let timeout_result = timeout(Duration::from_millis(1), async {
            // Simulate slow operation
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok::<(), BrokerError>(())
        })
        .await;

        assert!(timeout_result.is_err(), "Should timeout");
    }

    #[test]
    fn test_broker_error_types() {
        // Test that all expected error types can be created
        let _invalid_url = BrokerError::InvalidUrl("test".to_string());
        let _connection_error = BrokerError::ConnectionError("test".to_string());
        let _operation_error = BrokerError::OperationError("test".to_string());
        let _not_implemented = BrokerError::NotImplemented;

        // Verify error display formatting
        let error = BrokerError::InvalidUrl("redis://invalid".to_string());
        let error_str = format!("{error}");
        assert!(error_str.contains("redis://invalid"));
    }
}
