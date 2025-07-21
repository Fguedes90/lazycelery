use lazycelery::app::App;
use lazycelery::models::{Task, TaskStatus};
use lazycelery::ui::modals::{draw_confirmation_dialog, draw_help, draw_task_details_modal};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

mod test_broker_utils;
use test_broker_utils::MockBrokerBuilder;

#[test]
fn test_modal_content_generation() {
    let broker = MockBrokerBuilder::empty().build();
    let mut app = App::new(broker);

    // Test confirmation dialog setup
    app.show_confirmation = true;
    app.confirmation_message = "Are you sure you want to purge this queue?".to_string();
    assert!(app.show_confirmation);
    assert!(!app.confirmation_message.is_empty());

    // Test task details setup
    let test_task = Task {
        id: "test-task-123".to_string(),
        name: "test.task".to_string(),
        status: TaskStatus::Success,
        worker: Some("worker@host".to_string()),
        timestamp: chrono::Utc::now(),
        args: "[\"arg1\", \"arg2\"]".to_string(),
        kwargs: "{\"key\": \"value\"}".to_string(),
        result: Some("Task completed successfully".to_string()),
        traceback: None,
    };

    app.selected_task_details = Some(test_task.clone());
    app.show_task_details = true;
    assert!(app.show_task_details);
    assert!(app.selected_task_details.is_some());

    if let Some(task) = &app.selected_task_details {
        assert_eq!(task.id, "test-task-123");
        assert_eq!(task.name, "test.task");
        assert_eq!(task.status, TaskStatus::Success);
    }
}

#[test]
fn test_modal_state_transitions() {
    let broker = MockBrokerBuilder::empty().build();
    let mut app = App::new(broker);

    // Test initial state
    assert!(!app.show_help);
    assert!(!app.show_confirmation);
    assert!(!app.show_task_details);
    assert!(app.selected_task_details.is_none());

    // Test help modal
    app.show_help = true;
    assert!(app.show_help);

    // Test confirmation modal
    app.show_help = false;
    app.show_confirmation = true;
    app.confirmation_message = "Test confirmation".to_string();
    assert!(!app.show_help);
    assert!(app.show_confirmation);

    // Test task details modal
    app.show_confirmation = false;
    app.show_task_details = true;
    let task = Task {
        id: "task-456".to_string(),
        name: "another.task".to_string(),
        status: TaskStatus::Failure,
        worker: Some("worker2@host".to_string()),
        timestamp: chrono::Utc::now(),
        args: "[]".to_string(),
        kwargs: "{}".to_string(),
        result: None,
        traceback: Some(
            "Traceback (most recent call last):\n  File \"test.py\", line 1\nError: Test error"
                .to_string(),
        ),
    };
    app.selected_task_details = Some(task);

    assert!(!app.show_confirmation);
    assert!(app.show_task_details);
    assert!(app.selected_task_details.is_some());
}

#[test]
fn test_task_details_with_failure_traceback() {
    let broker = MockBrokerBuilder::empty().build();
    let mut app = App::new(broker);

    let failed_task = Task {
        id: "failed-task".to_string(),
        name: "failing.task".to_string(),
        status: TaskStatus::Failure,
        worker: Some("worker@host".to_string()),
        timestamp: chrono::Utc::now(),
        args: "[\"failed_arg\"]".to_string(),
        kwargs: "{\"debug\": true}".to_string(),
        result: None,
        traceback: Some("Traceback (most recent call last):\n  File \"worker.py\", line 42, in execute\n    raise ValueError(\"Test failure\")\nValueError: Test failure".to_string()),
    };

    app.selected_task_details = Some(failed_task.clone());
    app.show_task_details = true;

    if let Some(task) = &app.selected_task_details {
        assert_eq!(task.status, TaskStatus::Failure);
        assert!(task.traceback.is_some());

        if let Some(traceback) = &task.traceback {
            assert!(traceback.contains("ValueError"));
            assert!(traceback.contains("Test failure"));
            assert!(traceback.lines().count() > 1);
        }
    }
}

#[test]
fn test_task_details_various_statuses() {
    let broker = MockBrokerBuilder::empty().build();
    let mut app = App::new(broker);

    let statuses = vec![
        TaskStatus::Success,
        TaskStatus::Failure,
        TaskStatus::Pending,
        TaskStatus::Active,
        TaskStatus::Retry,
        TaskStatus::Revoked,
    ];

    for status in statuses {
        let task = Task {
            id: format!("task-{status:?}").to_lowercase(),
            name: format!("test.{status:?}").to_lowercase(),
            status: status.clone(),
            worker: Some("test-worker".to_string()),
            timestamp: chrono::Utc::now(),
            args: "[]".to_string(),
            kwargs: "{}".to_string(),
            result: if status == TaskStatus::Success {
                Some("OK".to_string())
            } else {
                None
            },
            traceback: if status == TaskStatus::Failure {
                Some("Error occurred".to_string())
            } else {
                None
            },
        };

        app.selected_task_details = Some(task.clone());
        assert_eq!(app.selected_task_details.as_ref().unwrap().status, status);
    }
}

#[test]
fn test_confirmation_dialog_messages() {
    let broker = MockBrokerBuilder::empty().build();
    let mut app = App::new(broker);

    let test_messages = vec![
        "Are you sure you want to purge the queue 'celery'?",
        "Confirm retry task 'test-task-123'?",
        "Revoke task 'failing-task-456'? This action cannot be undone.",
        "Delete all completed tasks?",
    ];

    for message in test_messages {
        app.confirmation_message = message.to_string();
        app.show_confirmation = true;

        assert_eq!(app.confirmation_message, message);
        assert!(app.show_confirmation);

        // Reset for next iteration
        app.show_confirmation = false;
    }
}

#[test]
fn test_modal_priority_logic() {
    let broker = MockBrokerBuilder::empty().build();
    let mut app = App::new(broker);

    // Multiple modals active - should follow priority order in UI rendering
    app.show_help = true;
    app.show_confirmation = true;
    app.show_task_details = true;

    // All can be true simultaneously in app state
    assert!(app.show_help);
    assert!(app.show_confirmation);
    assert!(app.show_task_details);

    // The actual priority is handled in the rendering functions
    // This test verifies the state can handle multiple modal flags
}

#[test]
fn test_task_details_edge_cases() {
    let broker = MockBrokerBuilder::empty().build();
    let mut app = App::new(broker);

    // Task with minimal data
    let minimal_task = Task {
        id: "minimal".to_string(),
        name: "minimal.task".to_string(),
        status: TaskStatus::Pending,
        worker: None, // No worker assigned
        timestamp: chrono::Utc::now(),
        args: "".to_string(),   // Empty args
        kwargs: "".to_string(), // Empty kwargs
        result: None,
        traceback: None,
    };

    app.selected_task_details = Some(minimal_task.clone());

    if let Some(task) = &app.selected_task_details {
        assert!(task.worker.is_none());
        assert!(task.args.is_empty());
        assert!(task.kwargs.is_empty());
        assert!(task.result.is_none());
        assert!(task.traceback.is_none());
    }

    // Task with very long data
    let long_task = Task {
        id: "x".repeat(100),
        name: "very.long.task.name.with.many.segments".to_string(),
        status: TaskStatus::Active,
        worker: Some("worker-with-very-long-hostname@example.domain.com".to_string()),
        timestamp: chrono::Utc::now(),
        args: format!("[{}]", "\"arg\", ".repeat(50)),
        kwargs: format!("{{{}}}", "\"key\": \"value\", ".repeat(20)),
        result: Some(
            "Very long result text that might wrap across multiple lines in the UI".to_string(),
        ),
        traceback: None,
    };

    app.selected_task_details = Some(long_task.clone());

    if let Some(task) = &app.selected_task_details {
        assert!(task.id.len() == 100);
        assert!(task.name.contains("very.long"));
        assert!(task.args.len() > 100);
        assert!(task.kwargs.len() > 100);
    }
}

// Integration test to verify modal rendering doesn't crash
#[test]
fn test_modal_rendering_integration() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    let broker = MockBrokerBuilder::empty().build();
    let mut app = App::new(broker);

    // Test rendering each modal type
    terminal
        .draw(|f| {
            app.show_help = true;
            draw_help(f);
        })
        .unwrap();

    app.show_help = false;
    app.show_confirmation = true;
    app.confirmation_message = "Test confirmation".to_string();

    terminal
        .draw(|f| {
            draw_confirmation_dialog(f, &app);
        })
        .unwrap();

    app.show_confirmation = false;
    app.show_task_details = true;
    app.selected_task_details = Some(Task {
        id: "test".to_string(),
        name: "test.task".to_string(),
        status: TaskStatus::Success,
        worker: Some("worker".to_string()),
        timestamp: chrono::Utc::now(),
        args: "[]".to_string(),
        kwargs: "{}".to_string(),
        result: Some("OK".to_string()),
        traceback: None,
    });

    terminal
        .draw(|f| {
            draw_task_details_modal(f, &app);
        })
        .unwrap();
}
