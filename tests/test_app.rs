use lazycelery::app::{App, Tab};
use lazycelery::models::{Queue, Task, TaskStatus, Worker, WorkerStatus};

mod test_broker_utils;
use test_broker_utils::MockBrokerBuilder;

#[test]
fn test_app_creation() {
    let broker = MockBrokerBuilder::empty().build();
    let app = App::new(broker);

    assert_eq!(app.selected_tab, Tab::Workers);
    assert!(!app.should_quit);
    assert_eq!(app.selected_worker, 0);
    assert_eq!(app.selected_task, 0);
    assert_eq!(app.selected_queue, 0);
    assert!(!app.show_help);
    assert!(!app.is_searching);
    assert_eq!(app.search_query, "");
}

#[test]
fn test_tab_navigation() {
    let broker = MockBrokerBuilder::empty().build();
    let mut app = App::new(broker);

    assert_eq!(app.selected_tab, Tab::Workers);

    app.next_tab();
    assert_eq!(app.selected_tab, Tab::Queues);

    app.next_tab();
    assert_eq!(app.selected_tab, Tab::Tasks);

    app.next_tab();
    assert_eq!(app.selected_tab, Tab::Workers);

    app.previous_tab();
    assert_eq!(app.selected_tab, Tab::Tasks);

    app.previous_tab();
    assert_eq!(app.selected_tab, Tab::Queues);

    app.previous_tab();
    assert_eq!(app.selected_tab, Tab::Workers);
}

#[tokio::test]
async fn test_app_refresh_data() {
    let test_workers = vec![Worker {
        hostname: "worker-1".to_string(),
        status: WorkerStatus::Online,
        concurrency: 4,
        queues: vec!["default".to_string()],
        active_tasks: vec![],
        processed: 100,
        failed: 5,
    }];

    let test_tasks = vec![Task {
        id: "task-1".to_string(),
        name: "send_email".to_string(),
        args: "[]".to_string(),
        kwargs: "{}".to_string(),
        status: TaskStatus::Success,
        worker: Some("worker-1".to_string()),
        timestamp: chrono::Utc::now(),
        result: None,
        traceback: None,
    }];

    let test_queues = vec![Queue {
        name: "default".to_string(),
        length: 10,
        consumers: 2,
    }];

    let broker = MockBrokerBuilder::new()
        .with_workers(test_workers.clone())
        .with_tasks(test_tasks.clone())
        .with_queues(test_queues.clone())
        .build();

    let mut app = App::new(broker);

    app.refresh_data().await.unwrap();

    assert_eq!(app.workers.len(), 1);
    assert_eq!(app.tasks.len(), 1);
    assert_eq!(app.queues.len(), 1);
    assert_eq!(app.workers[0].hostname, "worker-1");
    assert_eq!(app.tasks[0].id, "task-1");
    assert_eq!(app.queues[0].name, "default");
}

#[test]
fn test_item_selection() {
    let broker = MockBrokerBuilder::new()
        .with_workers(vec![
            Worker {
                hostname: "worker-1".to_string(),
                status: WorkerStatus::Online,
                concurrency: 4,
                queues: vec![],
                active_tasks: vec![],
                processed: 0,
                failed: 0,
            },
            Worker {
                hostname: "worker-2".to_string(),
                status: WorkerStatus::Online,
                concurrency: 4,
                queues: vec![],
                active_tasks: vec![],
                processed: 0,
                failed: 0,
            },
        ])
        .build();

    let mut app = App::new(broker);
    app.workers = vec![
        Worker {
            hostname: "worker-1".to_string(),
            status: WorkerStatus::Online,
            concurrency: 4,
            queues: vec![],
            active_tasks: vec![],
            processed: 0,
            failed: 0,
        },
        Worker {
            hostname: "worker-2".to_string(),
            status: WorkerStatus::Online,
            concurrency: 4,
            queues: vec![],
            active_tasks: vec![],
            processed: 0,
            failed: 0,
        },
    ];

    assert_eq!(app.selected_worker, 0);

    app.select_next();
    assert_eq!(app.selected_worker, 1);

    app.select_next();
    assert_eq!(app.selected_worker, 0); // Wraps around

    app.select_previous();
    assert_eq!(app.selected_worker, 1); // Wraps around

    app.select_previous();
    assert_eq!(app.selected_worker, 0);
}

#[test]
fn test_help_toggle() {
    let broker = MockBrokerBuilder::empty().build();
    let mut app = App::new(broker);

    assert!(!app.show_help);

    app.toggle_help();
    assert!(app.show_help);

    app.toggle_help();
    assert!(!app.show_help);
}

#[test]
fn test_search_functionality() {
    let broker = MockBrokerBuilder::empty().build();
    let mut app = App::new(broker);
    app.tasks = vec![
        Task {
            id: "abc123".to_string(),
            name: "send_email".to_string(),
            args: "[]".to_string(),
            kwargs: "{}".to_string(),
            status: TaskStatus::Success,
            worker: None,
            timestamp: chrono::Utc::now(),
            result: None,
            traceback: None,
        },
        Task {
            id: "def456".to_string(),
            name: "process_data".to_string(),
            args: "[]".to_string(),
            kwargs: "{}".to_string(),
            status: TaskStatus::Success,
            worker: None,
            timestamp: chrono::Utc::now(),
            result: None,
            traceback: None,
        },
    ];

    assert!(!app.is_searching);
    assert_eq!(app.search_query, "");

    app.start_search();
    assert!(app.is_searching);
    assert_eq!(app.search_query, "");

    app.search_query = "email".to_string();
    let filtered = app.get_filtered_tasks();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "send_email");

    app.search_query = "abc".to_string();
    let filtered = app.get_filtered_tasks();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, "abc123");

    app.stop_search();
    assert!(!app.is_searching);
    assert_eq!(app.search_query, "");
    assert_eq!(app.get_filtered_tasks().len(), 2);
}

#[test]
fn test_empty_state_selection() {
    let broker = MockBrokerBuilder::empty().build();
    let mut app = App::new(broker);

    // Should not panic when selecting with empty lists
    app.select_next();
    app.select_previous();

    assert_eq!(app.selected_worker, 0);
    assert_eq!(app.selected_task, 0);
    assert_eq!(app.selected_queue, 0);
}
