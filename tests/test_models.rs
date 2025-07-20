use lazycelery::models::{Worker, WorkerStatus, Task, TaskStatus, Queue};
use chrono::Utc;

#[test]
fn test_worker_creation() {
    let worker = Worker {
        hostname: "test-worker".to_string(),
        status: WorkerStatus::Online,
        concurrency: 4,
        queues: vec!["default".to_string()],
        active_tasks: vec![],
        processed: 100,
        failed: 5,
    };

    assert_eq!(worker.hostname, "test-worker");
    assert_eq!(worker.status, WorkerStatus::Online);
    assert_eq!(worker.concurrency, 4);
    assert_eq!(worker.utilization(), 0.0);
}

#[test]
fn test_worker_utilization() {
    let mut worker = Worker {
        hostname: "test-worker".to_string(),
        status: WorkerStatus::Online,
        concurrency: 4,
        queues: vec![],
        active_tasks: vec!["task1".to_string(), "task2".to_string()],
        processed: 0,
        failed: 0,
    };

    assert_eq!(worker.utilization(), 50.0);

    worker.active_tasks.push("task3".to_string());
    worker.active_tasks.push("task4".to_string());
    assert_eq!(worker.utilization(), 100.0);

    // Test edge case: zero concurrency
    worker.concurrency = 0;
    assert_eq!(worker.utilization(), 0.0);
}

#[test]
fn test_worker_serialization() {
    let worker = Worker {
        hostname: "worker-1".to_string(),
        status: WorkerStatus::Online,
        concurrency: 2,
        queues: vec!["queue1".to_string()],
        active_tasks: vec![],
        processed: 50,
        failed: 2,
    };

    let json = serde_json::to_string(&worker).unwrap();
    let deserialized: Worker = serde_json::from_str(&json).unwrap();
    
    assert_eq!(worker.hostname, deserialized.hostname);
    assert_eq!(worker.processed, deserialized.processed);
}

#[test]
fn test_task_creation() {
    let task = Task {
        id: "abc123".to_string(),
        name: "send_email".to_string(),
        args: "[]".to_string(),
        kwargs: "{}".to_string(),
        status: TaskStatus::Active,
        worker: Some("worker-1".to_string()),
        timestamp: Utc::now(),
        result: None,
        traceback: None,
    };

    assert_eq!(task.id, "abc123");
    assert_eq!(task.name, "send_email");
    assert_eq!(task.status, TaskStatus::Active);
}

#[test]
fn test_task_duration() {
    let past_time = Utc::now() - chrono::Duration::minutes(5);
    let task = Task {
        id: "test".to_string(),
        name: "test_task".to_string(),
        args: "[]".to_string(),
        kwargs: "{}".to_string(),
        status: TaskStatus::Success,
        worker: None,
        timestamp: past_time,
        result: None,
        traceback: None,
    };

    let duration = task.duration_since(Utc::now());
    assert!(duration.num_minutes() >= 4);
    assert!(duration.num_minutes() <= 6);
}

#[test]
fn test_task_serialization() {
    let task = Task {
        id: "123".to_string(),
        name: "process_data".to_string(),
        args: "[1, 2, 3]".to_string(),
        kwargs: r#"{"key": "value"}"#.to_string(),
        status: TaskStatus::Failure,
        worker: Some("worker-2".to_string()),
        timestamp: Utc::now(),
        result: Some("error result".to_string()),
        traceback: Some("traceback here".to_string()),
    };

    let json = serde_json::to_string(&task).unwrap();
    let deserialized: Task = serde_json::from_str(&json).unwrap();
    
    assert_eq!(task.id, deserialized.id);
    assert_eq!(task.status, deserialized.status);
    assert_eq!(task.traceback, deserialized.traceback);
}

#[test]
fn test_queue_creation() {
    let queue = Queue {
        name: "default".to_string(),
        length: 42,
        consumers: 3,
    };

    assert_eq!(queue.name, "default");
    assert_eq!(queue.length, 42);
    assert!(!queue.is_empty());
    assert!(queue.has_consumers());
}

#[test]
fn test_queue_empty_state() {
    let queue = Queue {
        name: "empty".to_string(),
        length: 0,
        consumers: 0,
    };

    assert!(queue.is_empty());
    assert!(!queue.has_consumers());
}

#[test]
fn test_queue_serialization() {
    let queue = Queue {
        name: "priority".to_string(),
        length: 100,
        consumers: 5,
    };

    let json = serde_json::to_string(&queue).unwrap();
    let deserialized: Queue = serde_json::from_str(&json).unwrap();
    
    assert_eq!(queue.name, deserialized.name);
    assert_eq!(queue.length, deserialized.length);
    assert_eq!(queue.consumers, deserialized.consumers);
}

#[test]
fn test_task_status_variants() {
    let statuses = vec![
        TaskStatus::Pending,
        TaskStatus::Active,
        TaskStatus::Success,
        TaskStatus::Failure,
        TaskStatus::Retry,
        TaskStatus::Revoked,
    ];

    for status in statuses {
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: TaskStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }
}
