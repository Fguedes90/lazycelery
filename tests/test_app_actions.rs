use lazycelery::app::{AppState, Tab};
use lazycelery::models::{Queue, Task, TaskStatus, Worker, WorkerStatus};

mod test_broker_utils;
use test_broker_utils::MockBrokerBuilder;

#[tokio::test]
async fn test_refresh_data_success() {
    let test_workers = vec![Worker {
        hostname: "test-host".to_string(),
        status: WorkerStatus::Online,
        concurrency: 4,
        queues: vec!["default".to_string()],
        active_tasks: vec!["task1".to_string(), "task2".to_string()],
        processed: 100,
        failed: 5,
    }];

    let test_tasks = vec![Task {
        id: "task1".to_string(),
        name: "test.task".to_string(),
        status: TaskStatus::Success,
        worker: Some("worker1".to_string()),
        timestamp: chrono::Utc::now(),
        args: "[]".to_string(),
        kwargs: "{}".to_string(),
        result: Some("OK".to_string()),
        traceback: None,
    }];

    let test_queues = vec![Queue {
        name: "default".to_string(),
        length: 5,
        consumers: 2,
    }];

    let broker = MockBrokerBuilder::new()
        .with_workers(test_workers.clone())
        .with_tasks(test_tasks.clone())
        .with_queues(test_queues.clone())
        .build();

    let mut app_state = AppState::new(broker);

    // Verify initial state is empty
    assert!(app_state.workers.is_empty());
    assert!(app_state.tasks.is_empty());
    assert!(app_state.queues.is_empty());

    // Refresh data
    let result = app_state.refresh_data().await;
    assert!(result.is_ok());

    // Verify data was loaded
    assert_eq!(app_state.workers.len(), 1);
    assert_eq!(app_state.tasks.len(), 1);
    assert_eq!(app_state.queues.len(), 1);

    assert_eq!(app_state.workers[0].hostname, "test-host");
    assert_eq!(app_state.tasks[0].id, "task1");
    assert_eq!(app_state.queues[0].name, "default");
}

#[tokio::test]
async fn test_refresh_data_selections_validation() {
    let broker = MockBrokerBuilder::empty().build();
    let mut app_state = AppState::new(broker);

    // Set selections beyond bounds
    app_state.selected_worker = 10;
    app_state.selected_task = 10;
    app_state.selected_queue = 10;

    let result = app_state.refresh_data().await;
    assert!(result.is_ok());

    // Selections should remain unchanged when no data is available
    // (validation only acts when data exists but selection is out of bounds)
    assert_eq!(app_state.selected_worker, 10);
    assert_eq!(app_state.selected_task, 10);
    assert_eq!(app_state.selected_queue, 10);
}

#[tokio::test]
async fn test_execute_pending_action_purge_queue() {
    let broker = MockBrokerBuilder::empty().build();
    let mut app_state = AppState::new(broker);

    // Add a queue and initiate purge action
    app_state.queues = vec![Queue {
        name: "test_queue".to_string(),
        length: 10,
        consumers: 1,
    }];
    app_state.selected_tab = Tab::Queues;
    app_state.selected_queue = 0;

    // Initiate purge action
    app_state.initiate_purge_queue();

    // Should show confirmation dialog
    assert!(app_state.show_confirmation);
    assert!(app_state.pending_action.is_some());

    // Execute the pending action
    let result = app_state.execute_pending_action().await;
    assert!(result.is_ok());

    // Pending action should be cleared
    assert!(app_state.pending_action.is_none());

    // Confirmation dialog should be hidden
    assert!(!app_state.show_confirmation);

    // Status message should be set
    assert!(!app_state.status_message.is_empty());
    assert!(app_state.status_message.contains("test_queue"));
}

#[tokio::test]
async fn test_execute_pending_action_retry_task() {
    let broker = MockBrokerBuilder::empty().build();
    let mut app_state = AppState::new(broker);

    // Add a task and initiate retry action
    app_state.tasks = vec![Task {
        id: "task123".to_string(),
        name: "test.task".to_string(),
        status: TaskStatus::Failure,
        worker: Some("worker1".to_string()),
        timestamp: chrono::Utc::now(),
        args: "[]".to_string(),
        kwargs: "{}".to_string(),
        result: None,
        traceback: Some("Error".to_string()),
    }];
    app_state.selected_tab = Tab::Tasks;
    app_state.selected_task = 0;

    // Initiate retry action
    app_state.initiate_retry_task();

    // Should show confirmation dialog
    assert!(app_state.show_confirmation);
    assert!(app_state.pending_action.is_some());

    // Execute the pending action
    let result = app_state.execute_pending_action().await;
    assert!(result.is_ok());

    assert!(app_state.pending_action.is_none());
    assert!(!app_state.show_confirmation);
    assert!(!app_state.status_message.is_empty());
    assert!(app_state.status_message.contains("task123"));
    assert!(app_state.status_message.contains("retry"));
}

#[tokio::test]
async fn test_execute_pending_action_revoke_task() {
    let broker = MockBrokerBuilder::empty().build();
    let mut app_state = AppState::new(broker);

    // Add a task and initiate revoke action
    app_state.tasks = vec![Task {
        id: "task456".to_string(),
        name: "test.task".to_string(),
        status: TaskStatus::Active,
        worker: Some("worker1".to_string()),
        timestamp: chrono::Utc::now(),
        args: "[]".to_string(),
        kwargs: "{}".to_string(),
        result: None,
        traceback: None,
    }];
    app_state.selected_tab = Tab::Tasks;
    app_state.selected_task = 0;

    // Initiate revoke action
    app_state.initiate_revoke_task();

    // Should show confirmation dialog
    assert!(app_state.show_confirmation);
    assert!(app_state.pending_action.is_some());

    // Execute the pending action
    let result = app_state.execute_pending_action().await;
    assert!(result.is_ok());

    assert!(app_state.pending_action.is_none());
    assert!(!app_state.show_confirmation);
    assert!(!app_state.status_message.is_empty());
    assert!(app_state.status_message.contains("task456"));
    assert!(app_state.status_message.contains("revoked"));
}

#[tokio::test]
async fn test_execute_pending_action_no_action() {
    let broker = MockBrokerBuilder::empty().build();
    let mut app_state = AppState::new(broker);

    // No pending action
    assert!(app_state.pending_action.is_none());

    let result = app_state.execute_pending_action().await;
    assert!(result.is_ok());

    // State should remain unchanged
    assert!(app_state.pending_action.is_none());
    assert!(!app_state.show_confirmation);
}

#[test]
fn test_initiate_purge_queue() {
    let broker = MockBrokerBuilder::empty().build();
    let mut app_state = AppState::new(broker);

    // Add a queue
    app_state.queues = vec![Queue {
        name: "celery".to_string(),
        length: 42,
        consumers: 3,
    }];

    app_state.selected_tab = Tab::Queues;
    app_state.selected_queue = 0;

    app_state.initiate_purge_queue();

    // Confirmation dialog should be shown
    assert!(app_state.show_confirmation);
    assert!(!app_state.confirmation_message.is_empty());
    assert!(app_state.confirmation_message.contains("celery"));
    assert!(app_state.confirmation_message.contains("42"));

    // Pending action should be set
    assert!(app_state.pending_action.is_some());
}

#[test]
fn test_initiate_purge_queue_wrong_tab() {
    let broker = MockBrokerBuilder::empty().build();
    let mut app_state = AppState::new(broker);

    app_state.queues = vec![Queue {
        name: "test".to_string(),
        length: 1,
        consumers: 1,
    }];

    app_state.selected_tab = Tab::Workers; // Wrong tab
    app_state.initiate_purge_queue();

    // Should not initiate purge
    assert!(!app_state.show_confirmation);
    assert!(app_state.pending_action.is_none());
}

#[test]
fn test_initiate_purge_queue_no_queues() {
    let broker = MockBrokerBuilder::empty().build();
    let mut app_state = AppState::new(broker);

    app_state.selected_tab = Tab::Queues;
    // No queues available

    app_state.initiate_purge_queue();

    // Should not initiate purge
    assert!(!app_state.show_confirmation);
    assert!(app_state.pending_action.is_none());
}

#[test]
fn test_initiate_retry_task() {
    let broker = MockBrokerBuilder::empty().build();
    let mut app_state = AppState::new(broker);

    // Add a task
    app_state.tasks = vec![Task {
        id: "retry-task".to_string(),
        name: "test.retry".to_string(),
        status: TaskStatus::Failure,
        worker: Some("worker1".to_string()),
        timestamp: chrono::Utc::now(),
        args: "[]".to_string(),
        kwargs: "{}".to_string(),
        result: None,
        traceback: Some("Error occurred".to_string()),
    }];

    app_state.selected_tab = Tab::Tasks;
    app_state.selected_task = 0;

    app_state.initiate_retry_task();

    // Confirmation dialog should be shown
    assert!(app_state.show_confirmation);
    assert!(!app_state.confirmation_message.is_empty());
    assert!(app_state.confirmation_message.contains("retry-task"));
    assert!(app_state.confirmation_message.contains("retry"));

    // Pending action should be set
    assert!(app_state.pending_action.is_some());
}

#[test]
fn test_initiate_revoke_task() {
    let broker = MockBrokerBuilder::empty().build();
    let mut app_state = AppState::new(broker);

    // Add a task
    app_state.tasks = vec![Task {
        id: "revoke-task".to_string(),
        name: "test.revoke".to_string(),
        status: TaskStatus::Active,
        worker: Some("worker1".to_string()),
        timestamp: chrono::Utc::now(),
        args: "[]".to_string(),
        kwargs: "{}".to_string(),
        result: None,
        traceback: None,
    }];

    app_state.selected_tab = Tab::Tasks;
    app_state.selected_task = 0;

    app_state.initiate_revoke_task();

    // Confirmation dialog should be shown
    assert!(app_state.show_confirmation);
    assert!(!app_state.confirmation_message.is_empty());
    assert!(app_state.confirmation_message.contains("revoke-task"));
    assert!(app_state.confirmation_message.contains("revoke"));

    // Pending action should be set
    assert!(app_state.pending_action.is_some());
}

#[test]
fn test_initiate_task_actions_wrong_tab() {
    let broker = MockBrokerBuilder::empty().build();
    let mut app_state = AppState::new(broker);

    app_state.tasks = vec![Task {
        id: "test-task".to_string(),
        name: "test".to_string(),
        status: TaskStatus::Active,
        worker: None,
        timestamp: chrono::Utc::now(),
        args: "[]".to_string(),
        kwargs: "{}".to_string(),
        result: None,
        traceback: None,
    }];

    app_state.selected_tab = Tab::Workers; // Wrong tab

    app_state.initiate_retry_task();
    assert!(!app_state.show_confirmation);
    assert!(app_state.pending_action.is_none());

    app_state.initiate_revoke_task();
    assert!(!app_state.show_confirmation);
    assert!(app_state.pending_action.is_none());
}

#[test]
fn test_initiate_task_actions_no_tasks() {
    let broker = MockBrokerBuilder::empty().build();
    let mut app_state = AppState::new(broker);

    app_state.selected_tab = Tab::Tasks;
    // No tasks available

    app_state.initiate_retry_task();
    assert!(!app_state.show_confirmation);
    assert!(app_state.pending_action.is_none());

    app_state.initiate_revoke_task();
    assert!(!app_state.show_confirmation);
    assert!(app_state.pending_action.is_none());
}

#[test]
fn test_initiate_task_actions_out_of_bounds() {
    let broker = MockBrokerBuilder::empty().build();
    let mut app_state = AppState::new(broker);

    app_state.tasks = vec![Task {
        id: "single-task".to_string(),
        name: "test".to_string(),
        status: TaskStatus::Active,
        worker: None,
        timestamp: chrono::Utc::now(),
        args: "[]".to_string(),
        kwargs: "{}".to_string(),
        result: None,
        traceback: None,
    }];

    app_state.selected_tab = Tab::Tasks;
    app_state.selected_task = 5; // Out of bounds

    app_state.initiate_retry_task();
    assert!(!app_state.show_confirmation);
    assert!(app_state.pending_action.is_none());

    app_state.initiate_revoke_task();
    assert!(!app_state.show_confirmation);
    assert!(app_state.pending_action.is_none());
}
