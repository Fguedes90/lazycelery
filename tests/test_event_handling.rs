use chrono::Utc;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazycelery::app::{App, Tab};
use lazycelery::models::{Task, TaskStatus, Worker, WorkerStatus};
use lazycelery::ui::events::{handle_key_event, AppEvent};

mod test_broker_utils;
use test_broker_utils::MockBrokerBuilder;

fn create_test_app() -> App {
    // Use consolidated mock broker with custom test data for navigation
    let broker = MockBrokerBuilder::new()
        .with_workers(vec![
            Worker {
                hostname: "worker-1".to_string(),
                status: WorkerStatus::Online,
                concurrency: 4,
                queues: vec!["default".to_string()],
                active_tasks: vec![],
                processed: 100,
                failed: 5,
            },
            Worker {
                hostname: "worker-2".to_string(),
                status: WorkerStatus::Offline,
                concurrency: 8,
                queues: vec!["celery".to_string()],
                active_tasks: vec![],
                processed: 250,
                failed: 12,
            },
        ])
        .with_tasks(vec![
            Task {
                id: "task-1".to_string(),
                name: "myapp.tasks.process_data".to_string(),
                args: "[]".to_string(),
                kwargs: "{}".to_string(),
                status: TaskStatus::Success,
                worker: Some("worker-1".to_string()),
                timestamp: Utc::now(),
                result: None,
                traceback: None,
            },
            Task {
                id: "task-2".to_string(),
                name: "myapp.tasks.another_task".to_string(),
                args: "[]".to_string(),
                kwargs: "{}".to_string(),
                status: TaskStatus::Failure,
                worker: Some("worker-2".to_string()),
                timestamp: Utc::now(),
                result: None,
                traceback: None,
            },
        ])
        .with_queues(vec![
            lazycelery::models::Queue {
                name: "default".to_string(),
                length: 10,
                consumers: 2,
            },
            lazycelery::models::Queue {
                name: "priority".to_string(),
                length: 5,
                consumers: 1,
            },
        ])
        .with_not_implemented_operations()
        .build();

    App::new(broker)
}

fn create_key_event(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

#[test]
fn test_quit_key_handling() {
    let mut app = create_test_app();
    assert!(!app.should_quit);

    handle_key_event(create_key_event(KeyCode::Char('q')), &mut app);
    assert!(app.should_quit);
}

#[test]
fn test_help_toggle() {
    let mut app = create_test_app();
    assert!(!app.show_help);

    // Toggle help on
    handle_key_event(create_key_event(KeyCode::Char('?')), &mut app);
    assert!(app.show_help);

    // Any key should toggle help off when it's showing
    handle_key_event(create_key_event(KeyCode::Char('a')), &mut app);
    assert!(!app.show_help);
}

#[test]
fn test_tab_navigation() {
    let mut app = create_test_app();
    assert_eq!(app.selected_tab, Tab::Workers);

    // Forward tab navigation
    handle_key_event(create_key_event(KeyCode::Tab), &mut app);
    assert_eq!(app.selected_tab, Tab::Queues);

    handle_key_event(create_key_event(KeyCode::Tab), &mut app);
    assert_eq!(app.selected_tab, Tab::Tasks);

    handle_key_event(create_key_event(KeyCode::Tab), &mut app);
    assert_eq!(app.selected_tab, Tab::Workers); // Wrap around

    // Backward tab navigation
    handle_key_event(create_key_event(KeyCode::BackTab), &mut app);
    assert_eq!(app.selected_tab, Tab::Tasks);

    handle_key_event(create_key_event(KeyCode::BackTab), &mut app);
    assert_eq!(app.selected_tab, Tab::Queues);

    handle_key_event(create_key_event(KeyCode::BackTab), &mut app);
    assert_eq!(app.selected_tab, Tab::Workers); // Wrap around
}

#[test]
fn test_item_navigation_workers_tab() {
    let mut app = create_test_app();
    app.selected_tab = Tab::Workers;
    assert_eq!(app.selected_worker, 0);

    // Navigate down
    handle_key_event(create_key_event(KeyCode::Down), &mut app);
    assert_eq!(app.selected_worker, 1);

    // Navigate down at end (should wrap to beginning)
    handle_key_event(create_key_event(KeyCode::Down), &mut app);
    assert_eq!(app.selected_worker, 0); // Wraps around

    // Navigate up from beginning (should wrap to end)
    handle_key_event(create_key_event(KeyCode::Up), &mut app);
    assert_eq!(app.selected_worker, 1); // Wraps to last item
}

#[test]
fn test_item_navigation_vim_keys() {
    let mut app = create_test_app();
    app.selected_tab = Tab::Tasks;
    assert_eq!(app.selected_task, 0);

    // Test vim-style navigation with 'j' (down)
    handle_key_event(create_key_event(KeyCode::Char('j')), &mut app);
    assert_eq!(app.selected_task, 1);

    // Test vim-style navigation with 'k' (up)
    handle_key_event(create_key_event(KeyCode::Char('k')), &mut app);
    assert_eq!(app.selected_task, 0);
}

#[test]
fn test_item_navigation_queues_tab() {
    let mut app = create_test_app();
    app.selected_tab = Tab::Queues;
    assert_eq!(app.selected_queue, 0);

    handle_key_event(create_key_event(KeyCode::Down), &mut app);
    assert_eq!(app.selected_queue, 1);

    handle_key_event(create_key_event(KeyCode::Up), &mut app);
    assert_eq!(app.selected_queue, 0);
}

#[test]
fn test_search_mode_activation() {
    let mut app = create_test_app();
    assert!(!app.is_searching);
    assert!(app.search_query.is_empty());

    // Start search
    handle_key_event(create_key_event(KeyCode::Char('/')), &mut app);
    assert!(app.is_searching);
}

#[test]
fn test_search_mode_character_input() {
    let mut app = create_test_app();
    app.is_searching = true;

    // Add characters to search query
    handle_key_event(create_key_event(KeyCode::Char('t')), &mut app);
    handle_key_event(create_key_event(KeyCode::Char('e')), &mut app);
    handle_key_event(create_key_event(KeyCode::Char('s')), &mut app);
    handle_key_event(create_key_event(KeyCode::Char('t')), &mut app);

    assert_eq!(app.search_query, "test");
}

#[test]
fn test_search_mode_backspace() {
    let mut app = create_test_app();
    app.is_searching = true;
    app.search_query = "hello".to_string();

    // Remove characters with backspace
    handle_key_event(create_key_event(KeyCode::Backspace), &mut app);
    assert_eq!(app.search_query, "hell");

    handle_key_event(create_key_event(KeyCode::Backspace), &mut app);
    assert_eq!(app.search_query, "hel");

    // Backspace on empty string should not panic
    app.search_query.clear();
    handle_key_event(create_key_event(KeyCode::Backspace), &mut app);
    assert_eq!(app.search_query, "");
}

#[test]
fn test_search_mode_escape() {
    let mut app = create_test_app();
    app.is_searching = true;
    app.search_query = "test query".to_string();

    // Escape should exit search mode
    handle_key_event(create_key_event(KeyCode::Esc), &mut app);
    assert!(!app.is_searching);
    // Query should be preserved for re-use
}

#[test]
fn test_search_mode_enter() {
    let mut app = create_test_app();
    app.is_searching = true;
    app.search_query = "process".to_string();

    // Enter should exit search mode
    handle_key_event(create_key_event(KeyCode::Enter), &mut app);
    assert!(!app.is_searching);
}

#[test]
fn test_search_mode_blocks_other_keys() {
    let mut app = create_test_app();
    app.is_searching = true;
    let original_tab = app.selected_tab;
    let original_should_quit = app.should_quit;

    // Normal navigation keys should be ignored in search mode
    handle_key_event(create_key_event(KeyCode::Tab), &mut app);
    assert_eq!(app.selected_tab, original_tab);

    handle_key_event(create_key_event(KeyCode::Char('q')), &mut app);
    assert_eq!(app.should_quit, original_should_quit);
    // But 'q' should be added to search query
    assert!(app.search_query.contains('q'));

    handle_key_event(create_key_event(KeyCode::Up), &mut app);
    // Up arrow should be ignored in search mode, no navigation change expected
}

#[test]
fn test_help_mode_blocks_other_keys() {
    let mut app = create_test_app();
    app.show_help = true;
    let original_tab = app.selected_tab;
    let original_should_quit = app.should_quit;

    // All keys should toggle help off when help is showing
    handle_key_event(create_key_event(KeyCode::Tab), &mut app);
    assert!(!app.show_help);
    assert_eq!(app.selected_tab, original_tab); // Navigation should not occur

    app.show_help = true;
    handle_key_event(create_key_event(KeyCode::Char('q')), &mut app);
    assert!(!app.show_help);
    assert_eq!(app.should_quit, original_should_quit); // Quit should not occur
}

#[test]
fn test_key_event_precedence() {
    let mut app = create_test_app();

    // Help mode has highest precedence - any key calls toggle_help() which flips the state
    app.show_help = true;
    app.is_searching = false; // Need to test help precedence without search mode interference

    handle_key_event(create_key_event(KeyCode::Char('a')), &mut app);

    assert!(!app.show_help); // Help should be toggled off (true -> false)
    assert!(!app.is_searching); // Search mode should remain off

    // Search mode has next precedence
    app.show_help = false;
    app.is_searching = true;
    handle_key_event(create_key_event(KeyCode::Char('q')), &mut app);
    assert!(!app.should_quit); // Quit should not happen
    assert!(app.search_query.contains('q')); // Character should be added to query
}

#[test]
fn test_empty_data_navigation() {
    let broker = MockBrokerBuilder::empty().build();
    let mut app = App::new(broker); // Empty app

    // Navigation on empty data should not crash
    handle_key_event(create_key_event(KeyCode::Up), &mut app);
    handle_key_event(create_key_event(KeyCode::Down), &mut app);

    // Selected indices should remain at 0
    assert_eq!(app.selected_worker, 0);
    assert_eq!(app.selected_task, 0);
    assert_eq!(app.selected_queue, 0);
}

#[test]
fn test_app_event_types() {
    // Test that AppEvent variants can be created
    let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
    let _app_event_key = AppEvent::Key(key_event);
    let _app_event_tick = AppEvent::Tick;
    let _app_event_refresh = AppEvent::Refresh;
}

#[test]
fn test_key_modifiers_handling() {
    let mut app = create_test_app();

    // Test that keys with modifiers are handled - current implementation ignores modifiers
    let key_with_ctrl = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
    let key_with_shift = KeyEvent::new(KeyCode::Char('Q'), KeyModifiers::SHIFT);
    let key_normal = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);

    // Current implementation only checks KeyCode, not modifiers, so all 'q' chars quit
    handle_key_event(key_with_ctrl, &mut app);
    assert!(app.should_quit); // Actually quits because modifiers are ignored

    // Reset for next test
    app.should_quit = false;
    handle_key_event(key_with_shift, &mut app);
    assert!(!app.should_quit); // 'Q' is different from 'q' in KeyCode, so no quit

    // Reset for next test
    app.should_quit = false;
    handle_key_event(key_normal, &mut app);
    assert!(app.should_quit);
}

mod navigation_edge_cases {
    use super::*;

    #[test]
    fn test_navigation_bounds_checking() {
        let mut app = create_test_app();

        // Test with single item lists
        app.workers = vec![app.workers[0].clone()];
        app.tasks = vec![app.tasks[0].clone()];
        app.queues = vec![app.queues[0].clone()];

        // Navigation should not go out of bounds
        app.selected_tab = Tab::Workers;
        handle_key_event(create_key_event(KeyCode::Down), &mut app);
        assert_eq!(app.selected_worker, 0); // Should stay at 0

        app.selected_tab = Tab::Tasks;
        handle_key_event(create_key_event(KeyCode::Down), &mut app);
        assert_eq!(app.selected_task, 0);

        app.selected_tab = Tab::Queues;
        handle_key_event(create_key_event(KeyCode::Down), &mut app);
        assert_eq!(app.selected_queue, 0);
    }

    #[test]
    fn test_tab_cycling_consistency() {
        let mut app = create_test_app();
        let starting_tab = app.selected_tab;

        // Full forward cycle should return to start
        handle_key_event(create_key_event(KeyCode::Tab), &mut app);
        handle_key_event(create_key_event(KeyCode::Tab), &mut app);
        handle_key_event(create_key_event(KeyCode::Tab), &mut app);
        assert_eq!(app.selected_tab, starting_tab);

        // Full backward cycle should return to start
        handle_key_event(create_key_event(KeyCode::BackTab), &mut app);
        handle_key_event(create_key_event(KeyCode::BackTab), &mut app);
        handle_key_event(create_key_event(KeyCode::BackTab), &mut app);
        assert_eq!(app.selected_tab, starting_tab);
    }

    #[test]
    fn test_search_with_special_characters() {
        let mut app = create_test_app();
        app.is_searching = true;

        // Test various special characters
        let special_chars = vec![
            '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '-', '_', '=', '+',
        ];

        for ch in special_chars {
            app.search_query.clear();
            handle_key_event(create_key_event(KeyCode::Char(ch)), &mut app);
            assert_eq!(app.search_query, ch.to_string());
        }
    }

    #[test]
    fn test_rapid_key_sequence() {
        let mut app = create_test_app();

        // Rapid sequence of navigation keys
        let keys = vec![
            KeyCode::Down,
            KeyCode::Down,
            KeyCode::Up,
            KeyCode::Down,
            KeyCode::Tab,
            KeyCode::Down,
            KeyCode::Up,
            KeyCode::BackTab,
        ];

        for key in keys {
            handle_key_event(create_key_event(key), &mut app);
            // App should remain in consistent state
            assert!(app.selected_worker < app.workers.len() || app.workers.is_empty());
            assert!(app.selected_task < app.tasks.len() || app.tasks.is_empty());
            assert!(app.selected_queue < app.queues.len() || app.queues.is_empty());
        }
    }
}
