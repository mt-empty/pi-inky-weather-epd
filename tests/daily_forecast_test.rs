//! Test to verify that all 7 days of daily forecast are populated correctly
//!
//! ## The Bug (Issue #16)
//!
//! The old code had a timezone conversion bug combined with off-by-one indexing:
//! 1. Converted local midnight to UTC using `with_timezone(&Utc)` - creates offset DateTime
//! 2. When local time is ahead of UTC (e.g., Oct 26 in Melbourne = Oct 25 UTC),
//!    the "current day" (Oct 25) appears to be in the "past" relative to local Oct 26
//! 3. With `day_index` starting at 1 and matching indices 1-7, Oct 25 gets skipped
//! 4. Result: Only 6 days fill (Oct 26-31), day 7 slot remains "NA"
//!
//! ## The Fix
//!
//! Use `date_naive()` to compare dates without timezone conversion:
//! - `clock.now_utc().date_naive()` - gets UTC date as NaiveDate
//! - `forecast_date.date_naive()` - extracts date from DateTime<Utc>
//! - Start `day_index` at 0, use indices 0-6 for exactly 7 days
//! - Check `day_index < 7` instead of `< 8`
//!
//! ## How to Verify the Fix
//!
//! ```bash
//! # Test should PASS with fixed code:
//! RUN_MODE=test cargo test --test daily_forecast_seven_days_test
//!
//! # Test should FAIL with buggy code:
//! git checkout HEAD~1 src/dashboard/context.rs
//! RUN_MODE=test cargo test --test daily_forecast_seven_days_test
//! # Should see: "FAILED: Day 7 name is 'NA'"
//!
//! # Restore fixed code:
//! git checkout HEAD -- src/dashboard/context.rs
//! ```

use chrono::NaiveDate;
use pi_inky_weather_epd::{
    clock::FixedClock,
    dashboard::context::ContextBuilder,
    domain::models::{Astronomical, DailyForecast, Temperature},
    CONFIG,
};

/// Create mock daily forecast data with exactly 7 days starting from a given date
/// All dates are at midnight UTC (00:00:00Z) to match Open-Meteo API format
fn create_mock_daily_forecast(start_date: NaiveDate, num_days: usize) -> Vec<DailyForecast> {
    (0..num_days)
        .map(|i| {
            let date = start_date + chrono::Days::new(i as u64);
            // Create DateTime at midnight UTC - this is how Open-Meteo returns daily dates
            let datetime = date.and_hms_opt(0, 0, 0).unwrap().and_utc();

            DailyForecast {
                date: Some(datetime),
                temp_max: Some(Temperature::celsius(20.0 + i as f32)),
                temp_min: Some(Temperature::celsius(10.0 + i as f32)),
                precipitation: None,
                astronomical: Some(Astronomical {
                    sunrise_time: Some(datetime),
                    sunset_time: Some(datetime),
                }),
            }
        })
        .collect()
}

/// CRITICAL TEST: Verifies all 7 days are populated in timezone UTC+11 (Melbourne)
///
/// This test specifically targets the timezone conversion bug where:
/// - Clock time: Oct 25, 2025, 22:00 UTC = Oct 26, 9:00 AM Melbourne (UTC+11)
/// - API returns 7 days starting from Oct 25, 00:00 UTC
/// - Old buggy code converts local midnight (Oct 26) back to UTC incorrectly
/// - This causes later days to be filtered out as "too far in future"
///
/// With the OLD buggy code at 22:00 UTC (9 AM Melbourne next day):
/// - local_date_truncated = Oct 26, 00:00 Melbourne
/// - utc_converted_date = Oct 25, 13:00 UTC
/// - Check: `naive_date > utc_converted_date + 7 days` = `> Nov 1, 13:00 UTC`
/// - Oct 31, 00:00 UTC < Nov 1, 13:00 UTC ✓ passes
/// - BUT with day_index starting at 1, indices 1-7 are used
/// - Index 1 gets Oct 25 (today already passed by clock)
/// - So we're actually off by one and miss the last day
///
/// With the FIXED code:
/// - current_utc_time = Oct 25 (naive date at clock time)
/// - Starts from Oct 25, fills indices 0-6
/// - All 7 days fill correctly
#[test]
fn test_timezone_bug_causes_missing_seventh_day() {
    assert!(
        CONFIG.debugging.disable_weather_api_requests,
        "Test requires RUN_MODE=test environment variable"
    );

    // Fixed time: Oct 25, 2025, 22:00 UTC
    // In Melbourne (UTC+11): Oct 26, 2025, 9:00 AM (next day!)
    let clock =
        FixedClock::from_rfc3339("2025-10-25T22:00:00Z").expect("Failed to create fixed clock");

    // Create exactly 7 days of forecast data starting from Oct 25, 2025 at 00:00 UTC
    // This matches the Open-Meteo API format
    let start_date = NaiveDate::from_ymd_opt(2025, 10, 25).unwrap();
    let daily_forecast_data = create_mock_daily_forecast(start_date, 7);

    // Build context with the forecast data
    let mut builder = ContextBuilder::new();
    builder.with_daily_forecast_data(daily_forecast_data, &clock);

    let context = &builder.context;

    // Verify all 7 days are populated with expected values
    // With buggy code, only 6 days would be filled (day2-day6 with Oct 26-30),
    // and day7 would remain "NA" because Oct 31 never gets assigned

    // Day 2 = Sun (Oct 26) - max 21°, min 11°
    assert_eq!(context.day2_name, "Sun", "Day 2 should be Sunday (Oct 26)");
    assert_eq!(context.day2_maxtemp, "21", "Day 2 max should be 21°");
    assert_eq!(context.day2_mintemp, "11", "Day 2 min should be 11°");

    // Day 3 = Mon (Oct 27) - max 22°, min 12°
    assert_eq!(context.day3_name, "Mon", "Day 3 should be Monday (Oct 27)");
    assert_eq!(context.day3_maxtemp, "22", "Day 3 max should be 22°");
    assert_eq!(context.day3_mintemp, "12", "Day 3 min should be 12°");

    // Day 4 = Tue (Oct 28) - max 23°, min 13°
    assert_eq!(context.day4_name, "Tue", "Day 4 should be Tuesday (Oct 28)");
    assert_eq!(context.day4_maxtemp, "23", "Day 4 max should be 23°");
    assert_eq!(context.day4_mintemp, "13", "Day 4 min should be 13°");

    // Day 5 = Wed (Oct 29) - max 24°, min 14°
    assert_eq!(
        context.day5_name, "Wed",
        "Day 5 should be Wednesday (Oct 29)"
    );
    assert_eq!(context.day5_maxtemp, "24", "Day 5 max should be 24°");
    assert_eq!(context.day5_mintemp, "14", "Day 5 min should be 14°");

    // Day 6 = Thu (Oct 30) - max 25°, min 15°
    assert_eq!(
        context.day6_name, "Thu",
        "Day 6 should be Thursday (Oct 30)"
    );
    assert_eq!(context.day6_maxtemp, "25", "Day 6 max should be 25°");
    assert_eq!(context.day6_mintemp, "15", "Day 6 min should be 15°");

    // CRITICAL: Day 7 = Fri (Oct 31) - max 26°, min 16°
    // With buggy code: day7 remains "NA" because Oct 25 was filtered as "past",
    // leaving only 6 days (Oct 26-31) to fill 7 slots (indices 1-7 in old code)
    assert_eq!(
        context.day7_name, "Fri",
        "FAILED: Day 7 name should be 'Fri' but got '{}' - the timezone/indexing bug is present!",
        context.day7_name
    );
    assert_eq!(
        context.day7_maxtemp, "26",
        "FAILED: Day 7 max should be 26° but got '{}' - Oct 31 data missing!",
        context.day7_maxtemp
    );
    assert_eq!(
        context.day7_mintemp, "16",
        "FAILED: Day 7 min should be 16° but got '{}' - Oct 31 data missing!",
        context.day7_mintemp
    );
}
