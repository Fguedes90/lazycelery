use lazycelery::utils::formatting::{format_duration, format_timestamp, truncate_string};
use chrono::{Duration, TimeZone, Utc};

#[test]
fn test_format_duration() {
    let duration = Duration::seconds(45);
    assert_eq!(format_duration(duration), "00:45");
    
    let duration = Duration::minutes(5) + Duration::seconds(30);
    assert_eq!(format_duration(duration), "05:30");
    
    let duration = Duration::hours(2) + Duration::minutes(15) + Duration::seconds(45);
    assert_eq!(format_duration(duration), "02:15:45");
    
    let duration = Duration::hours(10) + Duration::minutes(5) + Duration::seconds(3);
    assert_eq!(format_duration(duration), "10:05:03");
}

#[test]
fn test_format_timestamp() {
    let timestamp = Utc.with_ymd_and_hms(2024, 1, 15, 14, 30, 45).unwrap();
    assert_eq!(format_timestamp(timestamp), "2024-01-15 14:30:45");
    
    let timestamp = Utc.with_ymd_and_hms(2023, 12, 31, 23, 59, 59).unwrap();
    assert_eq!(format_timestamp(timestamp), "2023-12-31 23:59:59");
}

#[test]
fn test_truncate_string() {
    assert_eq!(truncate_string("hello", 10), "hello");
    assert_eq!(truncate_string("hello world", 8), "hello...");
    assert_eq!(truncate_string("this is a very long string", 10), "this is...");
    assert_eq!(truncate_string("short", 5), "short");
    assert_eq!(truncate_string("exactly", 7), "exactly");
    
    // Edge case: max_len < 3
    assert_eq!(truncate_string("hello", 2), "...");
}
