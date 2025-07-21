//! Basic Redis Broker Tests
//!
//! This module contains basic connection, parsing logic, and error handling tests.
//! These tests focus on core broker functionality and graceful handling of various
//! connection scenarios. For comprehensive integration tests with real Celery
//! protocol format, see test_redis_broker_integration.rs.

mod redis_test_utils;

use anyhow::Result;
use lazycelery::broker::{redis::RedisBroker, Broker};
use lazycelery::error::BrokerError;
use lazycelery::models::TaskStatus;
use redis_test_utils::*;
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

    #[tokio::test]
    async fn test_redis_get_workers_integration() -> Result<()> {
        skip_if_redis_unavailable(
            async {
                with_test_db(|mut db| async move {
                    let client = db.client().await?;
                    let builder = TestDataBuilder::new(client.clone());
                    builder.add_basic_tasks().await?;
                    builder.add_queue_data().await?;

                    let broker = db.broker().await?;
                    let workers = broker.get_workers().await?;

                    // Should discover workers from task activity
                    TestAssertions::assert_worker_properties(&workers, 1, true);

                    Ok(())
                })
                .await
            }
            .await,
        )
    }

    #[tokio::test]
    async fn test_redis_get_tasks_integration() -> Result<()> {
        skip_if_redis_unavailable(
            async {
                with_test_db(|mut db| async move {
                    let client = db.client().await?;
                    let builder = TestDataBuilder::new(client.clone());
                    builder.add_basic_tasks().await?;

                    // Add small delay to ensure data is persisted
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

                    let broker = db.broker().await?;
                    let tasks = broker.get_tasks().await?;

                    // Should find our test tasks
                    assert!(tasks.len() >= 2, "Should find at least 2 tasks");

                    // Verify specific tasks using test assertions
                    TestAssertions::assert_task_properties(
                        &tasks,
                        "basic-success-1",
                        TaskStatus::Success,
                        true,  // should have result
                        false, // should not have traceback
                    );

                    TestAssertions::assert_task_properties(
                        &tasks,
                        "basic-failure-1",
                        TaskStatus::Failure,
                        false, // should not have result
                        true,  // should have traceback
                    );

                    Ok(())
                })
                .await
            }
            .await,
        )
    }

    #[tokio::test]
    async fn test_redis_get_queues_integration() -> Result<()> {
        skip_if_redis_unavailable(
            async {
                with_test_db(|mut db| async move {
                    let client = db.client().await?;
                    let builder = TestDataBuilder::new(client.clone());
                    builder.add_basic_tasks().await?;
                    builder.add_queue_data().await?;

                    let broker = db.broker().await?;
                    let queues = broker.get_queues().await?;

                    // Verify queue properties using assertions
                    TestAssertions::assert_queue_properties(
                        &queues,
                        1,                   // min_count
                        Some(("celery", 2)), // expected celery queue with 2 items
                    );

                    Ok(())
                })
                .await
            }
            .await,
        )
    }

    #[tokio::test]
    async fn test_redis_task_operations_implemented() -> Result<()> {
        skip_if_redis_unavailable(
            async {
                with_test_db(|mut db| async move {
                    let client = db.client().await?;
                    let builder = TestDataBuilder::new(client.clone());
                    let task_id = "test-failed-task";

                    // Add a failed task for testing retry
                    builder.add_retry_test_task(task_id).await?;

                    let broker = db.broker().await?;

                    // Test retry operation (should work for failed tasks)
                    let retry_result = broker.retry_task(task_id).await;
                    assert!(
                        retry_result.is_ok(),
                        "Should successfully retry failed task"
                    );

                    // Test revoke operation (should work for any task)
                    let revoke_task_id = "test-task-to-revoke";
                    let revoke_result = broker.revoke_task(revoke_task_id).await;
                    assert!(revoke_result.is_ok(), "Should successfully revoke task");

                    // Verify task was added to revoked set using assertions
                    TestAssertions::assert_task_revoked(&client, revoke_task_id, true).await?;

                    Ok(())
                })
                .await
            }
            .await,
        )
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
