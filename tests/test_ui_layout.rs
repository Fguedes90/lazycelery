use lazycelery::ui::layout::{centered_rect, create_main_layout};
use ratatui::layout::Rect;

mod test_broker_utils;

#[test]
fn test_create_main_layout() {
    let area = Rect::new(0, 0, 100, 50);
    let layout = create_main_layout(area);

    assert_eq!(layout.len(), 3);

    // Header should be 3 units high
    assert_eq!(layout[0].height, 3);
    assert_eq!(layout[0].x, 0);
    assert_eq!(layout[0].y, 0);
    assert_eq!(layout[0].width, 100);

    // Status bar should be 3 units high at bottom
    assert_eq!(layout[2].height, 3);
    assert_eq!(layout[2].x, 0);
    assert_eq!(layout[2].y, 47); // 50 - 3
    assert_eq!(layout[2].width, 100);

    // Main content should fill remaining space
    assert_eq!(layout[1].height, 44); // 50 - 3 - 3
    assert_eq!(layout[1].x, 0);
    assert_eq!(layout[1].y, 3);
    assert_eq!(layout[1].width, 100);
}

#[test]
fn test_create_main_layout_small_area() {
    let area = Rect::new(10, 5, 20, 10);
    let layout = create_main_layout(area);

    assert_eq!(layout.len(), 3);

    // Header
    assert_eq!(layout[0].height, 3);
    assert_eq!(layout[0].x, 10);
    assert_eq!(layout[0].y, 5);
    assert_eq!(layout[0].width, 20);

    // Status bar
    assert_eq!(layout[2].height, 3);
    assert_eq!(layout[2].x, 10);
    assert_eq!(layout[2].y, 12); // 5 + 10 - 3
    assert_eq!(layout[2].width, 20);

    // Main content (minimum height 0 due to constraint)
    assert_eq!(layout[1].height, 4); // 10 - 3 - 3
    assert_eq!(layout[1].x, 10);
    assert_eq!(layout[1].y, 8); // 5 + 3
    assert_eq!(layout[1].width, 20);
}

#[test]
fn test_centered_rect_50_percent() {
    let area = Rect::new(0, 0, 100, 50);
    let centered = centered_rect(50, 50, area);

    // Should be 50% of width and height, centered
    assert_eq!(centered.width, 50);
    assert_eq!(centered.height, 25);
    assert_eq!(centered.x, 25); // (100 - 50) / 2
    assert_eq!(centered.y, 13); // Actual ratatui layout calculation
}

#[test]
fn test_centered_rect_80_percent() {
    let area = Rect::new(0, 0, 100, 50);
    let centered = centered_rect(80, 70, area);

    // Should be 80% width, 70% height, centered
    assert_eq!(centered.width, 80);
    assert_eq!(centered.height, 35);
    assert_eq!(centered.x, 10); // (100 - 80) / 2
    assert_eq!(centered.y, 8); // Actual ratatui layout calculation
}

#[test]
fn test_centered_rect_with_offset() {
    let area = Rect::new(20, 10, 60, 30);
    let centered = centered_rect(50, 50, area);

    // Should respect the area's offset
    assert_eq!(centered.width, 30); // 50% of 60
    assert_eq!(centered.height, 15); // 50% of 30
    assert_eq!(centered.x, 35); // 20 + (60 - 30) / 2
    assert_eq!(centered.y, 18); // Actual ratatui layout calculation
}

// Note: get_key_hints is a private function in layout.rs
// Testing it indirectly through integration tests would be more appropriate
// Since it's mainly used in draw_status_bar function
