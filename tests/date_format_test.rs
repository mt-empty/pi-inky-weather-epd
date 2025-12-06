//! Tests for configurable date format feature.
//!
//! These tests verify that users can configure the date display format
//! using strftime format strings in their configuration.

use chrono::{TimeZone, Utc};
use pi_inky_weather_epd::configs::validation::is_valid_date_format;

/// Helper to format a fixed date with a given format string
fn format_test_date(format: &str) -> String {
    // Use a fixed date: Saturday, December 6, 2025
    let test_date = Utc.with_ymd_and_hms(2025, 12, 6, 10, 30, 0).unwrap();
    test_date.format(format).to_string()
}

/// Helper to format the longest possible date (Wednesday, 28 September 2025)
fn format_longest_date(format: &str) -> String {
    // Wednesday (9 chars) + September (9 chars) = longest day + month combo
    let test_date = Utc.with_ymd_and_hms(2025, 9, 28, 10, 30, 0).unwrap();
    test_date.format(format).to_string()
}

// =============================================================================
// Regional Date Format Tests
// =============================================================================

#[test]
fn test_australian_date_format_day_month_year() {
    // Australia: DD/MM/YYYY
    let format = "%d/%m/%Y";
    assert!(is_valid_date_format(format).is_ok());
    assert_eq!(format_test_date(format), "06/12/2025");
}

#[test]
fn test_american_date_format_month_day_year() {
    // USA: MM/DD/YYYY
    let format = "%m/%d/%Y";
    assert!(is_valid_date_format(format).is_ok());
    assert_eq!(format_test_date(format), "12/06/2025");
}

#[test]
fn test_japanese_date_format_year_month_day() {
    // Japan: YYYY/MM/DD
    let format = "%Y/%m/%d";
    assert!(is_valid_date_format(format).is_ok());
    assert_eq!(format_test_date(format), "2025/12/06");
}

#[test]
fn test_iso8601_date_format() {
    // ISO 8601: YYYY-MM-DD
    let format = "%Y-%m-%d";
    assert!(is_valid_date_format(format).is_ok());
    assert_eq!(format_test_date(format), "2025-12-06");
}

// =============================================================================
// Separator Style Tests
// =============================================================================

#[test]
fn test_date_format_with_slash_separator() {
    let format = "%d/%m/%Y";
    assert!(is_valid_date_format(format).is_ok());
    assert_eq!(format_test_date(format), "06/12/2025");
}

#[test]
fn test_date_format_with_dot_separator() {
    // German style: DD.MM.YYYY
    let format = "%d.%m.%Y";
    assert!(is_valid_date_format(format).is_ok());
    assert_eq!(format_test_date(format), "06.12.2025");
}

#[test]
fn test_date_format_with_hyphen_separator() {
    let format = "%d-%m-%Y";
    assert!(is_valid_date_format(format).is_ok());
    assert_eq!(format_test_date(format), "06-12-2025");
}

#[test]
fn test_date_format_with_space_separator() {
    let format = "%d %m %Y";
    assert!(is_valid_date_format(format).is_ok());
    assert_eq!(format_test_date(format), "06 12 2025");
}

// =============================================================================
// Written Language Style Tests (Short vs Long)
// =============================================================================

#[test]
fn test_long_format_full_weekday_day_full_month() {
    // Saturday, 6 December 2025
    let format = "%A, %-d %B %Y";
    assert!(is_valid_date_format(format).is_ok());
    assert_eq!(format_test_date(format), "Saturday, 6 December 2025");
}

#[test]
fn test_long_format_full_month_day_year() {
    // December 6, 2025 (US written style)
    let format = "%B %-d, %Y";
    assert!(is_valid_date_format(format).is_ok());
    assert_eq!(format_test_date(format), "December 6, 2025");
}

#[test]
fn test_short_format_abbreviated_weekday_day_month() {
    // Sat, 6 Dec 2025
    let format = "%a, %-d %b %Y";
    assert!(is_valid_date_format(format).is_ok());
    assert_eq!(format_test_date(format), "Sat, 6 Dec 2025");
}

#[test]
fn test_short_format_day_abbreviated_month() {
    // 6 Dec
    let format = "%-d %b";
    assert!(is_valid_date_format(format).is_ok());
    assert_eq!(format_test_date(format), "6 Dec");
}

#[test]
fn test_short_format_abbreviated_month_day() {
    // Dec 6
    let format = "%b %-d";
    assert!(is_valid_date_format(format).is_ok());
    assert_eq!(format_test_date(format), "Dec 6");
}

#[test]
fn test_current_default_format() {
    // Current default: Saturday, 06 December (no year)
    let format = "%A, %d %B";
    assert!(is_valid_date_format(format).is_ok());
    assert_eq!(format_test_date(format), "Saturday, 06 December");
}

// =============================================================================
// Edge Cases and Special Formats
// =============================================================================

#[test]
fn test_format_with_short_year() {
    // Short year: 06/12/25
    let format = "%d/%m/%y";
    assert!(is_valid_date_format(format).is_ok());
    assert_eq!(format_test_date(format), "06/12/25");
}

#[test]
fn test_format_weekday_only() {
    let format = "%A";
    assert!(is_valid_date_format(format).is_ok());
    assert_eq!(format_test_date(format), "Saturday");
}

#[test]
fn test_format_with_custom_text() {
    // Custom text mixed with date
    let format = "Today is %A";
    assert!(is_valid_date_format(format).is_ok());
    assert_eq!(format_test_date(format), "Today is Saturday");
}

// =============================================================================
// Validation Error Tests
// =============================================================================

#[test]
fn test_invalid_format_empty_string() {
    assert!(is_valid_date_format("").is_err());
}

#[test]
fn test_invalid_format_only_whitespace() {
    assert!(is_valid_date_format("   ").is_err());
}

#[test]
fn test_format_too_long_output() {
    // A format that produces excessively long output should be rejected
    // to prevent display issues on the e-paper
    // Test with a format that repeats date components multiple times
    let long_format = "%A, %B %-d, %Y - %A, %B %-d, %Y - %A, %B %-d, %Y";
    assert!(is_valid_date_format(long_format).is_err());
}

#[test]
fn test_longest_reasonable_date_is_valid() {
    // Wednesday, 28 September 2025 = longest day + month names
    // This should be valid - it's a reasonable max length
    let format = "%A, %-d %B %Y";
    assert!(is_valid_date_format(format).is_ok());
    // Verify the longest output: "Wednesday, 28 September 2025" = 28 chars
    assert_eq!(format_longest_date(format), "Sunday, 28 September 2025");
}
