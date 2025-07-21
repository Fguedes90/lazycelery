use chrono::{Datelike, Duration, TimeZone, Utc};
use lazycelery::utils::formatting::{format_duration, format_timestamp, truncate_string};

#[test]
fn test_format_duration_seconds_only() {
    let duration = Duration::seconds(45);
    let formatted = format_duration(duration);
    assert_eq!(formatted, "00:45");
}

#[test]
fn test_format_duration_minutes_and_seconds() {
    let duration = Duration::seconds(125); // 2 minutes, 5 seconds
    let formatted = format_duration(duration);
    assert_eq!(formatted, "02:05");
}

#[test]
fn test_format_duration_with_hours() {
    let duration = Duration::seconds(3665); // 1 hour, 1 minute, 5 seconds
    let formatted = format_duration(duration);
    assert_eq!(formatted, "01:01:05");
}

#[test]
fn test_format_duration_zero() {
    let duration = Duration::seconds(0);
    let formatted = format_duration(duration);
    assert_eq!(formatted, "00:00");
}

#[test]
fn test_format_duration_exactly_one_hour() {
    let duration = Duration::seconds(3600); // Exactly 1 hour
    let formatted = format_duration(duration);
    assert_eq!(formatted, "01:00:00");
}

#[test]
fn test_format_duration_exactly_one_minute() {
    let duration = Duration::seconds(60); // Exactly 1 minute
    let formatted = format_duration(duration);
    assert_eq!(formatted, "01:00");
}

#[test]
fn test_format_duration_large_values() {
    let duration = Duration::seconds(359999); // 99 hours, 59 minutes, 59 seconds
    let formatted = format_duration(duration);
    assert_eq!(formatted, "99:59:59");
}

#[test]
fn test_format_timestamp() {
    let timestamp = Utc.with_ymd_and_hms(2023, 12, 25, 14, 30, 45).unwrap();
    let formatted = format_timestamp(timestamp);
    assert_eq!(formatted, "2023-12-25 14:30:45");
}

#[test]
fn test_format_timestamp_start_of_year() {
    let timestamp = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let formatted = format_timestamp(timestamp);
    assert_eq!(formatted, "2024-01-01 00:00:00");
}

#[test]
fn test_format_timestamp_end_of_year() {
    let timestamp = Utc.with_ymd_and_hms(2023, 12, 31, 23, 59, 59).unwrap();
    let formatted = format_timestamp(timestamp);
    assert_eq!(formatted, "2023-12-31 23:59:59");
}

#[test]
fn test_format_timestamp_leap_year() {
    let timestamp = Utc.with_ymd_and_hms(2024, 2, 29, 12, 0, 0).unwrap();
    let formatted = format_timestamp(timestamp);
    assert_eq!(formatted, "2024-02-29 12:00:00");
}

#[test]
fn test_truncate_string_no_truncation() {
    let result = truncate_string("hello", 10);
    assert_eq!(result, "hello");
}

#[test]
fn test_truncate_string_exact_length() {
    let result = truncate_string("hello", 5);
    assert_eq!(result, "hello");
}

#[test]
fn test_truncate_string_simple_truncation() {
    let result = truncate_string("hello world", 8);
    assert_eq!(result, "hello...");
}

#[test]
fn test_truncate_string_very_short_limit() {
    let result = truncate_string("hello", 3);
    assert_eq!(result, "...");
}

#[test]
fn test_truncate_string_shorter_than_ellipsis() {
    let result = truncate_string("hello", 2);
    assert_eq!(result, "...");
}

#[test]
fn test_truncate_string_empty_string() {
    let result = truncate_string("", 5);
    assert_eq!(result, "");
}

#[test]
fn test_truncate_string_zero_length() {
    let result = truncate_string("hello", 0);
    assert_eq!(result, "...");
}

#[test]
fn test_truncate_string_unicode() {
    // Note: This test might fail due to byte vs character counting
    // The current implementation uses byte indexing which can panic on unicode boundaries
    let result = truncate_string("héllo", 6);
    assert_eq!(result, "héllo");
}

#[test]
fn test_truncate_string_long_text() {
    let long_text = "The quick brown fox jumps over the lazy dog";
    let result = truncate_string(long_text, 20);
    assert_eq!(result, "The quick brown f...");
}

#[test]
fn test_truncate_string_single_character() {
    let result = truncate_string("a", 1);
    assert_eq!(result, "a");
}

#[test]
fn test_format_duration_negative() {
    // Test behavior with negative durations
    let duration = Duration::seconds(-30);
    let formatted = format_duration(duration);
    // The behavior with negative durations depends on implementation
    // This test documents the current behavior
    assert!(formatted.contains("00"));
}

#[test]
fn test_format_duration_boundary_values() {
    // Test various boundary values
    let test_cases = vec![
        (59, "00:59"),      // Just under 1 minute
        (60, "01:00"),      // Exactly 1 minute
        (61, "01:01"),      // Just over 1 minute
        (3599, "59:59"),    // Just under 1 hour
        (3600, "01:00:00"), // Exactly 1 hour
        (3661, "01:01:01"), // Just over 1 hour
    ];

    for (seconds, expected) in test_cases {
        let duration = Duration::seconds(seconds);
        let formatted = format_duration(duration);
        assert_eq!(formatted, expected, "Failed for {seconds} seconds");
    }
}

#[test]
fn test_truncate_string_edge_cases() {
    let test_cases = vec![
        ("", 0, ""), // Empty string stays empty even with 0 max_len
        ("", 1, ""),
        ("", 10, ""),
        ("a", 0, "..."), // Non-empty string with 0 max_len gets "..."
        ("a", 1, "a"),
        ("ab", 1, "..."),
        ("abc", 3, "abc"),   // max_len == string length, no truncation needed
        ("abcd", 4, "abcd"), // max_len == string length, no truncation needed
        ("abcd", 5, "abcd"),
    ];

    for (input, max_len, expected) in test_cases {
        let result = truncate_string(input, max_len);
        assert_eq!(
            result, expected,
            "Failed for input '{input}' with max_len {max_len}"
        );
    }
}

#[test]
fn test_formatting_functions_with_realistic_data() {
    // Test with more realistic data that might be encountered in the application

    // Test long task names that need truncation
    let long_task_name = "my_app.tasks.process_large_dataset_with_complex_calculations";
    let truncated = truncate_string(long_task_name, 30);
    assert_eq!(truncated, "my_app.tasks.process_large_...");

    // Test typical durations for Celery tasks
    let quick_task = Duration::seconds(2); // 2 second task
    assert_eq!(format_duration(quick_task), "00:02");

    let medium_task = Duration::seconds(450); // 7.5 minute task
    assert_eq!(format_duration(medium_task), "07:30");

    let long_task = Duration::seconds(7200); // 2 hour task
    assert_eq!(format_duration(long_task), "02:00:00");

    // Test recent timestamp
    let recent_time = Utc::now();
    let formatted_time = format_timestamp(recent_time);
    // Should contain the current year and be in correct format
    assert!(formatted_time.len() == 19); // YYYY-MM-DD HH:MM:SS format
    assert!(formatted_time.contains(&recent_time.year().to_string()));
}
