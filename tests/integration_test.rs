use async_trait::async_trait;
use chrono::Utc;
use lazycelery::app::App;
use lazycelery::broker::Broker;
use lazycelery::error::BrokerError;
use lazycelery::models::{Queue, Task, TaskStatus, Worker, WorkerStatus};

// Integration test broker that returns realistic data
struct IntegrationTestBroker;

#[async_trait]
impl Broker for IntegrationTestBroker {
    async fn connect(_url: &str) -> Result<Self, BrokerError> {
        Ok(IntegrationTestBroker)
    }

    async fn get_workers(&self) -> Result<Vec<Worker>, BrokerError> {
        Ok(vec![
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
    }

    async fn get_tasks(&self) -> Result<Vec<Task>, BrokerError> {
        Ok(vec![
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
                name: "app.tasks.cleanup_old_files".to_string(),
                args: "[]".to_string(),
                kwargs: r#"{"days": 30}"#.to_string(),
                status: TaskStatus::Pending,
                worker: None,
                timestamp: Utc::now(),
                result: None,
                traceback: None,
            },
        ])
    }

    async fn get_queues(&self) -> Result<Vec<Queue>, BrokerError> {
        Ok(vec![
            Queue {
                name: "default".to_string(),
                length: 42,
                consumers: 2,
            },
            Queue {
                name: "priority".to_string(),
                length: 5,
                consumers: 2,
            },
            Queue {
                name: "emails".to_string(),
                length: 128,
                consumers: 1,
            },
            Queue {
                name: "background".to_string(),
                length: 0,
                consumers: 0,
            },
        ])
    }

    async fn retry_task(&self, _task_id: &str) -> Result<(), BrokerError> {
        Ok(())
    }

    async fn revoke_task(&self, _task_id: &str) -> Result<(), BrokerError> {
        Ok(())
    }
}

#[tokio::test]
async fn test_full_application_flow() {
    let broker = Box::new(IntegrationTestBroker);
    let mut app = App::new(broker);

    // Initial state
    assert_eq!(app.workers.len(), 0);
    assert_eq!(app.tasks.len(), 0);
    assert_eq!(app.queues.len(), 0);

    // Refresh data
    app.refresh_data().await.unwrap();

    // Verify data loaded
    assert_eq!(app.workers.len(), 3);
    assert_eq!(app.tasks.len(), 5);
    assert_eq!(app.queues.len(), 4);

    // Test worker details
    let online_workers: Vec<_> = app
        .workers
        .iter()
        .filter(|w| w.status == WorkerStatus::Online)
        .collect();
    assert_eq!(online_workers.len(), 2);

    // Test task filtering
    app.search_query = "email".to_string();
    let filtered = app.get_filtered_tasks();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "app.tasks.send_welcome_email");

    // Test queue status
    let empty_queues: Vec<_> = app.queues.iter().filter(|q| q.is_empty()).collect();
    assert_eq!(empty_queues.len(), 1);
    assert_eq!(empty_queues[0].name, "background");

    // Test worker utilization
    assert_eq!(app.workers[0].utilization(), 25.0); // 2 active out of 8
    assert_eq!(app.workers[1].utilization(), 0.0); // 0 active out of 8

    // Test task status counts
    let active_tasks = app
        .tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Active)
        .count();
    assert_eq!(active_tasks, 2);

    let failed_tasks = app
        .tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Failure)
        .count();
    assert_eq!(failed_tasks, 1);
}

#[tokio::test]
async fn test_navigation_and_selection() {
    let broker = Box::new(IntegrationTestBroker);
    let mut app = App::new(broker);

    app.refresh_data().await.unwrap();

    // Test tab navigation
    assert_eq!(app.selected_tab, lazycelery::app::Tab::Workers);
    app.next_tab();
    assert_eq!(app.selected_tab, lazycelery::app::Tab::Queues);
    app.next_tab();
    assert_eq!(app.selected_tab, lazycelery::app::Tab::Tasks);

    // Test item selection
    app.selected_tab = lazycelery::app::Tab::Workers;
    assert_eq!(app.selected_worker, 0);
    app.select_next();
    assert_eq!(app.selected_worker, 1);
    app.select_next();
    assert_eq!(app.selected_worker, 2);
    app.select_next();
    assert_eq!(app.selected_worker, 0); // Wraps around

    // Test queue selection
    app.selected_tab = lazycelery::app::Tab::Queues;
    app.selected_queue = 0;
    for _ in 0..4 {
        app.select_next();
    }
    assert_eq!(app.selected_queue, 0); // Should wrap around after 4 queues
}
