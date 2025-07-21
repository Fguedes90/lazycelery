//! Shared test broker utilities
//!
//! This module provides a consolidated MockBroker implementation with a fluent builder API
//! to replace the 3 duplicated implementations across test files. It follows the excellent
//! pattern established in `redis_test_utils.rs`.

use async_trait::async_trait;
use chrono::Utc;
use lazycelery::broker::Broker;
use lazycelery::error::BrokerError;
use lazycelery::models::{Queue, Task, TaskStatus, Worker, WorkerStatus};

/// Builder for configurable mock broker instances
#[derive(Default)]
pub struct MockBrokerBuilder {
    workers: Vec<Worker>,
    tasks: Vec<Task>,
    queues: Vec<Queue>,
    should_fail_operations: bool,
    should_return_not_implemented: bool,
}

impl MockBrokerBuilder {
    /// Create a new empty broker builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a broker that returns empty collections (for UI navigation tests)
    pub fn empty() -> Self {
        Self::new()
    }

    /// Create a broker with basic test data (for app state tests)
    pub fn with_basic_data() -> Self {
        Self::new()
            .with_workers(vec![
                Worker {
                    hostname: "test-worker-1".to_string(),
                    status: WorkerStatus::Online,
                    concurrency: 4,
                    queues: vec!["default".to_string()],
                    active_tasks: vec!["task-1".to_string()],
                    processed: 100,
                    failed: 5,
                },
                Worker {
                    hostname: "test-worker-2".to_string(),
                    status: WorkerStatus::Offline,
                    concurrency: 2,
                    queues: vec!["priority".to_string()],
                    active_tasks: vec![],
                    processed: 50,
                    failed: 2,
                },
            ])
            .with_tasks(vec![
                Task {
                    id: "task-1".to_string(),
                    name: "test.task.example".to_string(),
                    args: r#"["arg1", "arg2"]"#.to_string(),
                    kwargs: r#"{"key": "value"}"#.to_string(),
                    status: TaskStatus::Active,
                    worker: Some("test-worker-1".to_string()),
                    timestamp: Utc::now(),
                    result: None,
                    traceback: None,
                },
                Task {
                    id: "task-2".to_string(),
                    name: "test.task.completed".to_string(),
                    args: "[]".to_string(),
                    kwargs: "{}".to_string(),
                    status: TaskStatus::Success,
                    worker: Some("test-worker-1".to_string()),
                    timestamp: Utc::now() - chrono::Duration::minutes(5),
                    result: Some(r#"{"result": "success"}"#.to_string()),
                    traceback: None,
                },
            ])
            .with_queues(vec![
                Queue {
                    name: "default".to_string(),
                    length: 10,
                    consumers: 2,
                },
                Queue {
                    name: "priority".to_string(),
                    length: 5,
                    consumers: 1,
                },
            ])
    }

    /// Create a broker with realistic integration test data
    pub fn with_integration_data() -> Self {
        Self::new()
            .with_workers(vec![
                Worker {
                    hostname: "celery@worker-prod-1".to_string(),
                    status: WorkerStatus::Online,
                    concurrency: 8,
                    queues: vec![
                        "default".to_string(),
                        "priority".to_string(),
                        "emails".to_string(),
                    ],
                    active_tasks: vec!["task-001".to_string(), "task-002".to_string()],
                    processed: 15234,
                    failed: 23,
                },
                Worker {
                    hostname: "celery@worker-prod-2".to_string(),
                    status: WorkerStatus::Online,
                    concurrency: 8,
                    queues: vec!["default".to_string(), "priority".to_string()],
                    active_tasks: vec![],
                    processed: 14892,
                    failed: 19,
                },
                Worker {
                    hostname: "celery@worker-prod-3".to_string(),
                    status: WorkerStatus::Offline,
                    concurrency: 4,
                    queues: vec!["background".to_string()],
                    active_tasks: vec![],
                    processed: 8923,
                    failed: 5,
                },
            ])
            .with_tasks(vec![
                Task {
                    id: "task-001".to_string(),
                    name: "app.tasks.send_welcome_email".to_string(),
                    args: r#"["user@example.com"]"#.to_string(),
                    kwargs: r#"{"template": "welcome"}"#.to_string(),
                    status: TaskStatus::Active,
                    worker: Some("celery@worker-prod-1".to_string()),
                    timestamp: Utc::now() - chrono::Duration::minutes(2),
                    result: None,
                    traceback: None,
                },
                Task {
                    id: "task-002".to_string(),
                    name: "app.tasks.process_payment".to_string(),
                    args: r#"[100.50, "USD"]"#.to_string(),
                    kwargs: r#"{"user_id": 12345}"#.to_string(),
                    status: TaskStatus::Active,
                    worker: Some("celery@worker-prod-1".to_string()),
                    timestamp: Utc::now() - chrono::Duration::seconds(30),
                    result: None,
                    traceback: None,
                },
                Task {
                    id: "task-003".to_string(),
                    name: "app.tasks.generate_report".to_string(),
                    args: "[]".to_string(),
                    kwargs: r#"{"report_type": "monthly", "month": 12}"#.to_string(),
                    status: TaskStatus::Success,
                    worker: Some("celery@worker-prod-2".to_string()),
                    timestamp: Utc::now() - chrono::Duration::hours(1),
                    result: Some(r#"{"status": "completed", "rows": 1523}"#.to_string()),
                    traceback: None,
                },
                Task {
                    id: "task-004".to_string(),
                    name: "app.tasks.sync_inventory".to_string(),
                    args: "[]".to_string(),
                    kwargs: "{}".to_string(),
                    status: TaskStatus::Failure,
                    worker: Some("celery@worker-prod-2".to_string()),
                    timestamp: Utc::now() - chrono::Duration::minutes(15),
                    result: None,
                    traceback: Some("Traceback (most recent call last):\n  File \"tasks.py\", line 45\n    ConnectionError: Database timeout".to_string()),
                },
                Task {
                    id: "task-005".to_string(),
                    name: "app.tasks.cleanup_temp_files".to_string(),
                    args: "[]".to_string(),
                    kwargs: r#"{"older_than": "1d"}"#.to_string(),
                    status: TaskStatus::Pending,
                    worker: None,
                    timestamp: Utc::now(),
                    result: None,
                    traceback: None,
                },
            ])
            .with_queues(vec![
                Queue {
                    name: "default".to_string(),
                    length: 42,
                    consumers: 3,
                },
                Queue {
                    name: "priority".to_string(),
                    length: 8,
                    consumers: 2,
                },
                Queue {
                    name: "emails".to_string(),
                    length: 15,
                    consumers: 1,
                },
                Queue {
                    name: "background".to_string(),
                    length: 0,
                    consumers: 0,
                },
            ])
    }

    /// Add custom workers to the broker
    pub fn with_workers(mut self, workers: Vec<Worker>) -> Self {
        self.workers = workers;
        self
    }

    /// Add custom tasks to the broker
    pub fn with_tasks(mut self, tasks: Vec<Task>) -> Self {
        self.tasks = tasks;
        self
    }

    /// Add custom queues to the broker
    pub fn with_queues(mut self, queues: Vec<Queue>) -> Self {
        self.queues = queues;
        self
    }

    /// Configure broker to fail all operations (for error testing)
    pub fn with_failing_operations(mut self) -> Self {
        self.should_fail_operations = true;
        self
    }

    /// Configure broker to return NotImplemented for operations (for UI tests)
    pub fn with_not_implemented_operations(mut self) -> Self {
        self.should_return_not_implemented = true;
        self
    }

    /// Build the configured mock broker
    pub fn build(self) -> Box<dyn Broker> {
        Box::new(MockBroker {
            workers: self.workers,
            tasks: self.tasks,
            queues: self.queues,
            should_fail_operations: self.should_fail_operations,
            should_return_not_implemented: self.should_return_not_implemented,
        })
    }
}

/// Mock broker implementation with configurable behavior
struct MockBroker {
    workers: Vec<Worker>,
    tasks: Vec<Task>,
    queues: Vec<Queue>,
    should_fail_operations: bool,
    should_return_not_implemented: bool,
}

#[async_trait]
impl Broker for MockBroker {
    async fn connect(_url: &str) -> Result<Self, BrokerError> {
        // This should not be called directly - use MockBrokerBuilder instead
        Ok(MockBroker {
            workers: vec![],
            tasks: vec![],
            queues: vec![],
            should_fail_operations: false,
            should_return_not_implemented: false,
        })
    }

    async fn get_workers(&self) -> Result<Vec<Worker>, BrokerError> {
        if self.should_fail_operations {
            return Err(BrokerError::ConnectionError(
                "Simulated failure".to_string(),
            ));
        }
        Ok(self.workers.clone())
    }

    async fn get_tasks(&self) -> Result<Vec<Task>, BrokerError> {
        if self.should_fail_operations {
            return Err(BrokerError::ConnectionError(
                "Simulated failure".to_string(),
            ));
        }
        Ok(self.tasks.clone())
    }

    async fn get_queues(&self) -> Result<Vec<Queue>, BrokerError> {
        if self.should_fail_operations {
            return Err(BrokerError::ConnectionError(
                "Simulated failure".to_string(),
            ));
        }
        Ok(self.queues.clone())
    }

    async fn retry_task(&self, _task_id: &str) -> Result<(), BrokerError> {
        if self.should_fail_operations {
            return Err(BrokerError::OperationError("Retry failed".to_string()));
        }
        if self.should_return_not_implemented {
            return Err(BrokerError::NotImplemented);
        }
        Ok(())
    }

    async fn revoke_task(&self, _task_id: &str) -> Result<(), BrokerError> {
        if self.should_fail_operations {
            return Err(BrokerError::OperationError("Revoke failed".to_string()));
        }
        if self.should_return_not_implemented {
            return Err(BrokerError::NotImplemented);
        }
        Ok(())
    }

    async fn purge_queue(&self, _queue_name: &str) -> Result<u64, BrokerError> {
        if self.should_fail_operations {
            return Err(BrokerError::OperationError("Purge failed".to_string()));
        }
        if self.should_return_not_implemented {
            return Err(BrokerError::NotImplemented);
        }
        // Return simulated purge count
        Ok(42)
    }
}

/// Helper functions for common test scenarios
impl MockBrokerBuilder {
    /// Create a broker for UI navigation tests (empty data, NotImplemented operations)
    pub fn for_ui_tests() -> Box<dyn Broker> {
        Self::empty().with_not_implemented_operations().build()
    }

    /// Create a broker for app state tests (basic data, working operations)
    pub fn for_app_tests() -> Box<dyn Broker> {
        Self::with_basic_data().build()
    }

    /// Create a broker for integration tests (realistic data, working operations)
    pub fn for_integration_tests() -> Box<dyn Broker> {
        Self::with_integration_data().build()
    }

    /// Create a broker for error testing (empty data, failing operations)
    pub fn for_error_tests() -> Box<dyn Broker> {
        Self::empty().with_failing_operations().build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_empty_broker() {
        let broker = MockBrokerBuilder::empty().build();

        let workers = broker.get_workers().await.unwrap();
        let tasks = broker.get_tasks().await.unwrap();
        let queues = broker.get_queues().await.unwrap();

        assert!(workers.is_empty());
        assert!(tasks.is_empty());
        assert!(queues.is_empty());
    }

    #[tokio::test]
    async fn test_basic_data_broker() {
        let broker = MockBrokerBuilder::with_basic_data().build();

        let workers = broker.get_workers().await.unwrap();
        let tasks = broker.get_tasks().await.unwrap();
        let queues = broker.get_queues().await.unwrap();

        assert_eq!(workers.len(), 2);
        assert_eq!(tasks.len(), 2);
        assert_eq!(queues.len(), 2);

        assert_eq!(workers[0].hostname, "test-worker-1");
        assert_eq!(tasks[0].id, "task-1");
        assert_eq!(queues[0].name, "default");
    }

    #[tokio::test]
    async fn test_integration_data_broker() {
        let broker = MockBrokerBuilder::with_integration_data().build();

        let workers = broker.get_workers().await.unwrap();
        let tasks = broker.get_tasks().await.unwrap();
        let queues = broker.get_queues().await.unwrap();

        assert_eq!(workers.len(), 3);
        assert_eq!(tasks.len(), 5);
        assert_eq!(queues.len(), 4);

        assert_eq!(workers[0].hostname, "celery@worker-prod-1");
        assert_eq!(tasks[0].id, "task-001");
        assert_eq!(queues[0].name, "default");
    }

    #[tokio::test]
    async fn test_not_implemented_operations() {
        let broker = MockBrokerBuilder::empty()
            .with_not_implemented_operations()
            .build();

        let result = broker.retry_task("test-task").await;
        assert!(matches!(result, Err(BrokerError::NotImplemented)));
    }

    #[tokio::test]
    async fn test_failing_operations() {
        let broker = MockBrokerBuilder::empty().with_failing_operations().build();

        let result = broker.retry_task("test-task").await;
        assert!(matches!(result, Err(BrokerError::OperationError(_))));
    }

    #[tokio::test]
    async fn test_convenience_constructors() {
        let ui_broker = MockBrokerBuilder::for_ui_tests();
        let app_broker = MockBrokerBuilder::for_app_tests();
        let integration_broker = MockBrokerBuilder::for_integration_tests();
        let error_broker = MockBrokerBuilder::for_error_tests();

        // Test that UI broker returns NotImplemented
        let result = ui_broker.retry_task("test").await;
        assert!(matches!(result, Err(BrokerError::NotImplemented)));

        // Test that app broker has data
        let workers = app_broker.get_workers().await.unwrap();
        assert_eq!(workers.len(), 2);

        // Test that integration broker has realistic data
        let tasks = integration_broker.get_tasks().await.unwrap();
        assert_eq!(tasks.len(), 5);

        // Test that error broker fails operations
        let result = error_broker.retry_task("test").await;
        assert!(matches!(result, Err(BrokerError::OperationError(_))));
    }
}
