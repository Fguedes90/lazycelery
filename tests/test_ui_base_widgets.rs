use lazycelery::ui::widgets::base::helpers::*;
use ratatui::style::{Color, Modifier, Style};

#[test]
fn test_selection_style() {
    let style = selection_style();

    assert_eq!(style.bg, Some(Color::DarkGray));
    assert!(style.add_modifier.contains(Modifier::BOLD));
}

#[test]
fn test_titled_block() {
    let _block = titled_block("Test Title");

    // Test that the function runs without panicking
    // The actual title format is " Test Title " (with spaces)
    // Function executed successfully if we reach this point
}

#[test]
fn test_titled_block_different_titles() {
    let test_titles = vec![
        "Workers",
        "Queues",
        "Tasks",
        "Details",
        "Very Long Title With Spaces",
        "",
        "Title with 123 numbers",
    ];

    for title in test_titles {
        let _block = titled_block(title);
        // Test that each call completes successfully
        // No assertion needed - function success is implicit
    }
}

#[test]
fn test_no_data_message() {
    let _paragraph = no_data_message("workers");

    // The paragraph is created successfully
    // We can't easily inspect the exact text content, but we can verify structure
    // The function should create a paragraph with a border and title

    // Test with different item types
    let item_types = vec!["workers", "tasks", "queues", "results"];

    for item_type in item_types {
        let _paragraph = no_data_message(item_type);
        // Each call should succeed without panicking
        // No assertion needed - function success is implicit
    }
}

#[test]
fn test_status_line() {
    let line = status_line("Status", "Online", Color::Green);

    // Verify the line contains both label and value spans
    assert_eq!(line.spans.len(), 2);

    // First span should be the label with colon
    assert_eq!(line.spans[0].content, "Status: ");

    // Second span should be the value with color
    assert_eq!(line.spans[1].content, "Online");
    assert_eq!(line.spans[1].style.fg, Some(Color::Green));
}

#[test]
fn test_status_line_different_colors() {
    let test_cases = vec![
        ("Active", "Running", Color::Green),
        ("Failed", "Error", Color::Red),
        ("Pending", "Waiting", Color::Yellow),
        ("Unknown", "N/A", Color::Gray),
    ];

    for (label, value, color) in test_cases {
        let line = status_line(label, value, color);

        assert_eq!(line.spans.len(), 2);
        assert_eq!(line.spans[0].content, format!("{label}: "));
        assert_eq!(line.spans[1].content, value);
        assert_eq!(line.spans[1].style.fg, Some(color));
    }
}

#[test]
fn test_field_line() {
    let line = field_line("Name", "test-worker");

    assert_eq!(line.spans.len(), 2);
    assert_eq!(line.spans[0].content, "Name: ");
    assert_eq!(line.spans[1].content, "test-worker");

    // Both spans should have default styling (no specific color)
    assert_eq!(line.spans[0].style, Style::default());
    assert_eq!(line.spans[1].style, Style::default());
}

#[test]
fn test_field_line_edge_cases() {
    let test_cases = vec![
        ("", ""),
        ("Empty Value", ""),
        ("", "Empty Label"),
        ("Long Label Name", "Short"),
        ("ID", "very-long-id-string-with-many-characters-123456789"),
        ("Special/Chars!", "Value@#$%^&*()"),
    ];

    for (label, value) in test_cases {
        let line = field_line(label, value);

        assert_eq!(line.spans.len(), 2);
        assert_eq!(line.spans[0].content, format!("{label}: "));
        assert_eq!(line.spans[1].content, value);
    }
}

#[test]
fn test_highlighted_field_line() {
    let line = highlighted_field_line("Priority", "High", Color::Red);

    assert_eq!(line.spans.len(), 2);
    assert_eq!(line.spans[0].content, "Priority: ");
    assert_eq!(line.spans[1].content, "High");
    assert_eq!(line.spans[1].style.fg, Some(Color::Red));

    // First span should be default style
    assert_eq!(line.spans[0].style, Style::default());
}

#[test]
fn test_highlighted_field_line_various_colors() {
    let test_cases = vec![
        ("Error", "Critical", Color::Red),
        ("Success", "Completed", Color::Green),
        ("Warning", "Attention", Color::Yellow),
        ("Info", "Details", Color::Blue),
        ("Debug", "Verbose", Color::Magenta),
    ];

    for (label, value, color) in test_cases {
        let line = highlighted_field_line(label, value, color);

        assert_eq!(line.spans.len(), 2);
        assert_eq!(line.spans[0].content, format!("{label}: "));
        assert_eq!(line.spans[1].content, value);
        assert_eq!(line.spans[1].style.fg, Some(color));
    }
}

#[test]
fn test_line_span_consistency() {
    // Test that all line creation functions produce consistent span structures

    let status_line_result = status_line("Test", "Value", Color::White);
    let field_line_result = field_line("Test", "Value");
    let highlighted_line_result = highlighted_field_line("Test", "Value", Color::White);

    // All should have exactly 2 spans
    assert_eq!(status_line_result.spans.len(), 2);
    assert_eq!(field_line_result.spans.len(), 2);
    assert_eq!(highlighted_line_result.spans.len(), 2);

    // All should have the same label format
    assert_eq!(status_line_result.spans[0].content, "Test: ");
    assert_eq!(field_line_result.spans[0].content, "Test: ");
    assert_eq!(highlighted_line_result.spans[0].content, "Test: ");

    // All should have the same value content
    assert_eq!(status_line_result.spans[1].content, "Value");
    assert_eq!(field_line_result.spans[1].content, "Value");
    assert_eq!(highlighted_line_result.spans[1].content, "Value");
}

#[test]
fn test_helper_functions_with_unicode() {
    // Test with Unicode characters to ensure proper handling
    let unicode_cases = vec![
        ("状态", "在线", Color::Green),
        ("Ñame", "Tëst", Color::Blue),
        ("🔧 Tool", "⚡ Status", Color::Yellow),
        ("Émoji", "🎉 Success", Color::Green),
    ];

    for (label, value, color) in unicode_cases {
        let status_line_result = status_line(label, value, color);
        let field_line_result = field_line(label, value);
        let highlighted_line_result = highlighted_field_line(label, value, color);

        // Should handle Unicode without issues
        assert_eq!(status_line_result.spans[0].content, format!("{label}: "));
        assert_eq!(status_line_result.spans[1].content, value);

        assert_eq!(field_line_result.spans[0].content, format!("{label}: "));
        assert_eq!(field_line_result.spans[1].content, value);

        assert_eq!(
            highlighted_line_result.spans[0].content,
            format!("{label}: ")
        );
        assert_eq!(highlighted_line_result.spans[1].content, value);
    }
}
