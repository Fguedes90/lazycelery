use lazycelery::app::App;

mod test_broker_utils;
use test_broker_utils::MockBrokerBuilder;

#[test]
fn test_navigation_and_selection() {
    let broker = MockBrokerBuilder::for_integration_tests();
    let mut app = App::new(broker);

    // Test initial state
    assert_eq!(app.selected_worker, 0);
    assert_eq!(app.selected_task, 0);
    assert_eq!(app.selected_queue, 0);

    // Test tab navigation
    app.next_tab();
    app.next_tab();

    // Test item selection
    app.select_next();
    app.select_previous();

    // Verify state is maintained
    assert_eq!(app.selected_worker, 0);
}

#[tokio::test]
async fn test_full_application_flow() {
    let broker = MockBrokerBuilder::for_integration_tests();
    let mut app = App::new(broker);

    // Test data refresh
    app.refresh_data().await.unwrap();

    // Verify we have the expected integration test data
    assert_eq!(app.workers.len(), 3);
    assert_eq!(app.tasks.len(), 5);
    assert_eq!(app.queues.len(), 4);

    // Verify specific worker data
    assert_eq!(app.workers[0].hostname, "celery@worker-prod-1");
    assert_eq!(app.workers[0].processed, 15234);

    // Verify specific task data
    assert_eq!(app.tasks[0].id, "task-001");
    assert_eq!(app.tasks[0].name, "app.tasks.send_welcome_email");

    // Verify specific queue data
    assert_eq!(app.queues[0].name, "default");
    assert_eq!(app.queues[0].length, 42);

    // Test navigation: Workers -> Queues -> Tasks -> Queues
    app.next_tab(); // Workers -> Queues
    app.next_tab(); // Queues -> Tasks
    app.previous_tab(); // Tasks -> Queues

    // Should be on Queues tab now, so test queue selection
    app.select_next();
    assert_eq!(app.selected_queue, 1);

    app.select_previous();
    assert_eq!(app.selected_queue, 0);

    // Go to Tasks tab to test task selection
    app.next_tab(); // Queues -> Tasks
    app.select_next();
    assert_eq!(app.selected_task, 1);

    app.select_previous();
    assert_eq!(app.selected_task, 0);

    // Go back to Workers tab to test worker selection
    app.next_tab(); // Tasks -> Workers
    app.select_next();
    assert_eq!(app.selected_worker, 1);

    app.select_previous();
    assert_eq!(app.selected_worker, 0);
}
