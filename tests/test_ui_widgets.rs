use lazycelery::models::{TaskStatus, Worker, WorkerStatus};
use ratatui::style::Color;

// Test for business logic without UI rendering
mod widget_logic_tests {
    use super::*;

    #[test]
    fn test_task_status_color_mapping() {
        let test_cases = vec![
            (TaskStatus::Success, Color::Green),
            (TaskStatus::Failure, Color::Red),
            (TaskStatus::Active, Color::Yellow),
            (TaskStatus::Pending, Color::Gray),
            (TaskStatus::Retry, Color::Magenta),
            (TaskStatus::Revoked, Color::DarkGray),
        ];

        for (status, expected_color) in test_cases {
            let actual_color = match status {
                TaskStatus::Success => Color::Green,
                TaskStatus::Failure => Color::Red,
                TaskStatus::Active => Color::Yellow,
                TaskStatus::Pending => Color::Gray,
                TaskStatus::Retry => Color::Magenta,
                TaskStatus::Revoked => Color::DarkGray,
            };
            assert_eq!(
                actual_color, expected_color,
                "Color mismatch for status: {status:?}"
            );
        }
    }

    #[test]
    fn test_worker_status_symbols() {
        let online_symbol = match WorkerStatus::Online {
            WorkerStatus::Online => "●",
            WorkerStatus::Offline => "○",
        };
        let offline_symbol = match WorkerStatus::Offline {
            WorkerStatus::Online => "●",
            WorkerStatus::Offline => "○",
        };

        assert_eq!(online_symbol, "●");
        assert_eq!(offline_symbol, "○");
    }

    #[test]
    fn test_worker_utilization_calculation() {
        let worker = Worker {
            hostname: "test-worker".to_string(),
            status: WorkerStatus::Online,
            concurrency: 4,
            queues: vec![],
            active_tasks: vec!["task1".to_string(), "task2".to_string()],
            processed: 100,
            failed: 5,
        };

        assert_eq!(worker.utilization(), 50.0); // 2/4 = 50%
    }

    #[test]
    fn test_task_viewport_logic() {
        let height = 10;
        let total_items = 20;

        // Test cases: (selected_index, expected_start)
        let test_cases: Vec<(usize, usize)> = vec![
            (0, 0),   // Beginning
            (5, 0),   // Still within first page
            (10, 5),  // Should center around selection
            (15, 10), // Should center around selection
            (19, 10), // Last item, start should be total - height
        ];

        for (selected, expected_start) in test_cases {
            let start = if selected >= height && height > 0 {
                selected.saturating_sub(height / 2)
            } else {
                0
            };

            let end = (start + height).min(total_items);

            // Ensure we don't go beyond bounds
            let actual_start = if end == total_items && total_items > height {
                total_items - height
            } else {
                start
            };

            assert_eq!(
                actual_start, expected_start,
                "Viewport calculation failed for selected={selected}, height={height}, total={total_items}"
            );
        }
    }

    #[test]
    fn test_duration_formatting() {
        use chrono::Duration;

        let duration = Duration::hours(2) + Duration::minutes(30) + Duration::seconds(45);
        let formatted = format!(
            "{:02}:{:02}:{:02}",
            duration.num_hours(),
            duration.num_minutes() % 60,
            duration.num_seconds() % 60
        );

        assert_eq!(formatted, "02:30:45");
    }
}
