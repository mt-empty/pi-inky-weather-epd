//! Test to verify that all 7 days of daily forecast are populated correctly
//!
//! ## The Bug (Issue #16)
//!
//! The old code had a timezone conversion bug:
//!
//! 1. The forecast date names where based on the date returned by the UTC date returned by the API
//! 2. However the time component return by open meteo is different than the one returned by BOM
//! 3. So when the date+time was converted to local time, the date could shift to the previous or next day
//!
//! ## The Fix
//!
//! 1. Get current local midnight date
//! 2. Pre-initialize day names from local calendar (tomorrow through +6 days) using the midnight date
//! 3. So now dates are based on local calendar days, and forecast data is matched accordingly
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
            // Create DateTime at 22:00 UTC to simulate API data with time component
            let datetime = date.and_hms_opt(22, 0, 0).unwrap().and_utc();

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
/// - API returns 7 days starting from Oct 25, 22:00 UTC
/// - Day names must match local calendar days (not UTC dates)
///
/// With the FIXED code at 22:00 UTC (9 AM Melbourne next day):
/// - local_midnight_time = Oct 26, 00:00 Melbourne
/// - utc_midnight_time = Oct 25, 13:00 UTC (local midnight converted to UTC)
/// - Day names pre-initialized: day2=Mon (Oct 27), day3=Tue (Oct 28), etc.
/// - Forecast dates >= Oct 25, 13:00 UTC are kept
/// - day_index 0 = today (Oct 25 22:00 UTC) for sunrise/sunset
/// - day_index 1-6 fill day2-day7 with temp/icon data
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

    // Create exactly 7 days of forecast data starting from Oct 25, 2025 at 22:00 UTC
    // This matches the Open-Meteo API format
    let start_date = NaiveDate::from_ymd_opt(2025, 10, 25).unwrap();
    let daily_forecast_data = create_mock_daily_forecast(start_date, 7); // Need 8 days for day_index 1-7 to work +1

    // Build context with the forecast data
    let mut builder = ContextBuilder::new();
    builder.with_daily_forecast_data(daily_forecast_data, &clock);

    let context = &builder.context;

    // Verify all 7 days are populated with expected values
    // Day names are pre-initialized from local calendar: tomorrow (Oct 27) through +6 days
    // day_index 0 = today (sunrise/sunset only), day_index 1-6 fill day2-day7

    // Day 2 (Oct 27 Mon) - day_index=1, forecast data index 1
    assert_eq!(context.day2_name, "Mon", "Day 2 should be Monday (Oct 27)");
    assert_eq!(context.day2_mintemp, "11", "Day 2 min temp should be 11");
    assert_eq!(context.day2_maxtemp, "21", "Day 2 max temp should be 21");

    // Day 3 (Oct 28 Tue) - day_index=2, forecast data index 2
    assert_eq!(context.day3_name, "Tue", "Day 3 should be Tuesday (Oct 28)");
    assert_eq!(context.day3_mintemp, "12", "Day 3 min temp should be 12");
    assert_eq!(context.day3_maxtemp, "22", "Day 3 max temp should be 22");

    // Day 4 (Oct 29 Wed) - day_index=3, forecast data index 3
    assert_eq!(
        context.day4_name, "Wed",
        "Day 4 should be Wednesday (Oct 29)"
    );
    assert_eq!(context.day4_mintemp, "13", "Day 4 min temp should be 13");
    assert_eq!(context.day4_maxtemp, "23", "Day 4 max temp should be 23");

    // Day 5 (Oct 30 Thu) - day_index=4, forecast data index 4
    assert_eq!(
        context.day5_name, "Thu",
        "Day 5 should be Thursday (Oct 30)"
    );
    assert_eq!(context.day5_mintemp, "14", "Day 5 min temp should be 14");
    assert_eq!(context.day5_maxtemp, "24", "Day 5 max temp should be 24");

    // Day 6 (Oct 31 Fri) - day_index=5, forecast data index 5
    assert_eq!(context.day6_name, "Fri", "Day 6 should be Friday (Oct 31)");
    assert_eq!(context.day6_mintemp, "15", "Day 6 min temp should be 15");
    assert_eq!(context.day6_maxtemp, "25", "Day 6 max temp should be 25");

    // Day 7 (Nov 1 Sat) - day_index=6, forecast data index 6
    assert_eq!(context.day7_name, "Sat", "Day 7 should be Saturday (Nov 1)");
    assert_eq!(context.day7_mintemp, "16", "Day 7 min temp should be 16");
    assert_eq!(context.day7_maxtemp, "26", "Day 7 max temp should be 26");
    // assert_eq!(context.day7_mintemp, "17");
    // assert_eq!(context.day7_maxtemp, "27");

    // CRITICAL: Verify day 7 is NOT "NA" (the bug would cause this)
    assert_ne!(
        context.day7_name, "NA",
        "FAILED: Day 7 name is 'NA' - timezone bug is present!"
    );
}
