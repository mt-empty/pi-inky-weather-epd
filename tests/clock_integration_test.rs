/// Integration tests demonstrating time-controlled testing with FixedClock
///
/// These tests show how to use the Clock trait abstraction to write
/// deterministic tests for time-dependent functionality.
use chrono::{Datelike, TimeZone, Timelike, Utc};
use pi_inky_weather_epd::clock::{Clock, FixedClock};

#[test]
fn test_fixed_clock_allows_time_controlled_testing() {
    // Arrange: Create a fixed point in time
    let test_time = Utc.with_ymd_and_hms(2025, 3, 15, 14, 30, 0).unwrap();
    let clock = FixedClock::new(test_time);

    // Act: Get time from clock
    let now = clock.now_utc();

    // Assert: Time is exactly what we set
    assert_eq!(now.year(), 2025);
    assert_eq!(now.month(), 3);
    assert_eq!(now.day(), 15);
    assert_eq!(now.hour(), 14);
    assert_eq!(now.minute(), 30);
}

#[test]
fn test_can_simulate_midnight_boundary() {
    // This demonstrates testing edge cases around midnight
    let midnight = Utc.with_ymd_and_hms(2025, 12, 31, 23, 59, 59).unwrap();
    let clock = FixedClock::new(midnight);

    let now = clock.now_utc();
    assert_eq!(now.year(), 2025);
    assert_eq!(now.month(), 12);
    assert_eq!(now.day(), 31);
    assert_eq!(now.hour(), 23);
    assert_eq!(now.minute(), 59);
    assert_eq!(now.second(), 59);
}

#[test]
fn test_can_simulate_different_seasons() {
    // Summer in Australia
    let summer = Utc.with_ymd_and_hms(2025, 1, 15, 12, 0, 0).unwrap();
    let summer_clock = FixedClock::new(summer);
    assert_eq!(summer_clock.now_utc().month(), 1);

    // Winter in Australia
    let winter = Utc.with_ymd_and_hms(2025, 7, 15, 12, 0, 0).unwrap();
    let winter_clock = FixedClock::new(winter);
    assert_eq!(winter_clock.now_utc().month(), 7);
}

#[test]
fn test_fixed_clock_from_rfc3339_string() {
    // Demonstrates creating FixedClock from ISO 8601 string
    // (useful for loading test fixtures from JSON)
    let clock = FixedClock::from_rfc3339("2025-06-21T10:30:00Z").unwrap();

    let now = clock.now_utc();
    assert_eq!(now.year(), 2025);
    assert_eq!(now.month(), 6);
    assert_eq!(now.day(), 21);
    assert_eq!(now.hour(), 10);
    assert_eq!(now.minute(), 30);
}

#[test]
fn test_multiple_calls_return_consistent_time() {
    // FixedClock guarantees time doesn't advance between calls
    let test_time = Utc.with_ymd_and_hms(2025, 10, 9, 22, 0, 0).unwrap();
    let clock = FixedClock::new(test_time);

    let time1 = clock.now_utc();
    // Simulate some processing time
    std::thread::sleep(std::time::Duration::from_millis(10));
    let time2 = clock.now_utc();

    // Time hasn't advanced - perfect for reproducible tests
    assert_eq!(time1, time2);
}

/// Example of how to test a function that depends on current time
///
/// In production code, you would:
/// 1. Accept `&dyn Clock` as parameter
/// 2. Call `clock.now_local()` or `clock.now_utc()` instead of `Local::now()`
/// 3. In tests, pass FixedClock
/// 4. In production, pass SystemClock
#[test]
fn test_time_dependent_function_example() {
    fn is_business_hours(clock: &dyn Clock) -> bool {
        let now = clock.now_utc();
        let hour = now.hour();
        (9..17).contains(&hour)
    }

    // Test during business hours
    let business_hour = Utc.with_ymd_and_hms(2025, 3, 15, 10, 0, 0).unwrap();
    let clock1 = FixedClock::new(business_hour);
    assert!(is_business_hours(&clock1));

    // Test outside business hours
    let after_hours = Utc.with_ymd_and_hms(2025, 3, 15, 20, 0, 0).unwrap();
    let clock2 = FixedClock::new(after_hours);
    assert!(!is_business_hours(&clock2));
}
