//! Test Open-Meteo date conversion to ensure proper timezone handling
//!
//! This test verifies that NaiveDate from Open-Meteo API gets converted to UTC timestamps
//! that produce correct local dates when used in ContextBuilder.
//!
//! Bug context: When current_time wraps at midnight boundaries (e.g., NY at 19:00 UTC,
//! Melbourne at early UTC), manual time arithmetic failed to adjust dates properly.
//! The fix uses DateTime arithmetic which handles day boundaries automatically.

use chrono::{NaiveDate, TimeZone, Utc};
use pi_inky_weather_epd::{
    apis::open_meteo::models::OpenMeteoHourlyResponse,
    clock::{Clock, FixedClock},
    dashboard::context::ContextBuilder,
    domain::models::DailyForecast,
};
use serial_test::serial;
use std::fs;

/// Load Open-Meteo fixture and convert to domain models
fn load_open_meteo_daily_forecasts(fixture_path: &str) -> Vec<DailyForecast> {
    let fixture_data =
        fs::read_to_string(fixture_path).expect("Failed to read Open-Meteo forecast fixture file");

    let response: OpenMeteoHourlyResponse =
        serde_json::from_str(&fixture_data).expect("Failed to deserialize Open-Meteo fixture");

    response.into()
}

/// Test NY 6PM EST (before GMT midnight) - dates should align with local calendar
///
/// **Scenario**: Dec 28, 2025 at 11PM GMT = Dec 28, 6PM EST
/// **Fixture dates**: Start with 2025-12-28 (same day in both timezones)
/// **Expected**: Dec 28 should be "today" in both GMT data and EST local time
#[test]
#[serial]
fn test_open_meteo_ny_6pm_before_gmt_midnight() {
    let original_tz = std::env::var("TZ").ok();
    unsafe { std::env::set_var("TZ", "America/New_York") };

    // Clock at 2025-12-28T23:00:00Z (11PM GMT = 6PM EST, still Dec 28 in both)
    let clock = FixedClock::new(Utc.with_ymd_and_hms(2025, 12, 28, 23, 0, 0).unwrap());

    let today_local = clock.now_local().date_naive();
    println!("\n=== 6PM EST Test ===");
    println!("Clock UTC: 2025-12-28T23:00:00Z (11PM GMT)");
    println!("Clock Local: {} (6PM EST)", clock.now_local());
    println!("Today (local): {}", today_local);

    // Load fixture captured at this time
    let daily_forecasts = load_open_meteo_daily_forecasts(
        "tests/fixtures/ny_6pm_before_gmt/open_meteo_forecast.json",
    );

    let mut context_builder = ContextBuilder::new();
    context_builder.with_daily_forecast_data(daily_forecasts.clone(), &clock);

    println!("\nAPI dates (already NaiveDate, no conversion needed):");
    for (i, forecast) in daily_forecasts.iter().enumerate().take(3) {
        if let Some(date) = forecast.date {
            println!("  Forecast[{}]: Date={}", i, date);
        }
    }

    let dates_in_context: Vec<NaiveDate> = daily_forecasts.iter().filter_map(|f| f.date).collect();

    println!("\nDates in context: {:?}", &dates_in_context[..3]);

    // Verify today (Dec 28) is present
    assert!(
        dates_in_context.contains(&today_local),
        "Today's date ({}) should be present in forecasts. Found: {:?}",
        today_local,
        &dates_in_context[..3]
    );

    // CRITICAL: Verify context output fields (what users actually see)
    let context = &context_builder.context;
    println!("\n=== Context Output Verification ===");
    println!(
        "Day 2 (tomorrow): {} - Max: {}",
        context.day2_name, context.day2_maxtemp
    );
    println!(
        "Sunrise: {} | Sunset: {}",
        context.sunrise_time, context.sunset_time
    );

    // Verify day names are populated (tomorrow should be Monday, Dec 29)
    assert_eq!(
        context.day2_name, "Mon",
        "Tomorrow (Dec 29, 2025) should be Monday"
    );

    // Verify temperature fields are populated (not "NA")
    assert_ne!(
        context.day2_maxtemp, "NA",
        "Tomorrow's max temp should be populated"
    );
    assert_ne!(
        context.day2_mintemp, "NA",
        "Tomorrow's min temp should be populated"
    );

    // Verify sunrise/sunset are populated (today's data exists)
    assert_ne!(
        context.sunrise_time, "NA",
        "Today's sunrise should be populated"
    );
    assert_ne!(
        context.sunset_time, "NA",
        "Today's sunset should be populated"
    );

    // Cleanup
    unsafe {
        match original_tz {
            Some(tz) => std::env::set_var("TZ", tz),
            None => std::env::remove_var("TZ"),
        }
    }
}

/// Test NY 7PM EST (after GMT midnight) - Shows data bucket issue
///
/// **Scenario**: Dec 29, 2025 at 12AM GMT = Dec 28, 7PM EST
/// **Issue**: Open-Meteo API returns dates starting Dec 29 (GMT's "today"),
///            but it's still Dec 28 in EST.
/// **Current Fix**: Dates are NaiveDate (no conversion). API's "2025-12-29" displays as Dec 29.
/// **Correct Solution**: API should include Dec 28 using `past_days=1` parameter.
///
/// This test **should fail** with the current fixture because the API doesn't include Dec 28.
/// The test documents the expected behavior once `past_days=1` is added to API requests.
#[test]
#[serial]
#[ignore = "Requires past_days=1 in API request to include yesterday"]
fn test_open_meteo_ny_7pm_after_gmt_midnight() {
    let original_tz = std::env::var("TZ").ok();
    unsafe { std::env::set_var("TZ", "America/New_York") };

    // Clock at 2025-12-29T00:00:00Z (midnight GMT = 7PM EST on Dec 28)
    let clock = FixedClock::new(Utc.with_ymd_and_hms(2025, 12, 29, 0, 0, 0).unwrap());

    let today_local = clock.now_local().date_naive();
    println!("\n=== 7PM EST Test (CRITICAL BUG CASE) ===");
    println!("Clock UTC: 2025-12-29T00:00:00Z (midnight GMT - next day!)");
    println!(
        "Clock Local: {} (7PM EST - still Dec 28!)",
        clock.now_local()
    );
    println!("Today (local): {}", today_local);

    // Load fixture captured at this time - dates start from Dec 29 (GMT's today)
    let daily_forecasts =
        load_open_meteo_daily_forecasts("tests/fixtures/ny_7pm_after_gmt/open_meteo_forecast.json");

    let mut context_builder = ContextBuilder::new();
    context_builder.with_daily_forecast_data(daily_forecasts.clone(), &clock);

    println!("\nAPI returns dates starting Dec 29 (GMT's today):");
    println!("After timezone conversion to EST:");
    for (i, forecast) in daily_forecasts.iter().enumerate().take(3) {
        if let Some(date) = forecast.date {
            // date is already NaiveDate
            println!("  Forecast[{}]: Date={}", i, date);
        }
    }

    let dates_in_context: Vec<NaiveDate> = daily_forecasts
        .iter()
        .filter_map(|f| f.date)
        .map(|dt| dt)
        .collect();

    println!("\nLocal dates in context: {:?}", &dates_in_context[..3]);

    // CRITICAL: Verify today (Dec 28 EST) is present, even though API returned Dec 29 GMT
    assert!(
        dates_in_context.contains(&today_local),
        "Today's date ({}) should be present after conversion. Found: {:?}",
        today_local,
        &dates_in_context[..3]
    );

    // Additional verification: First forecast date should be Dec 28 (today in EST)
    if let Some(first_forecast_date) = daily_forecasts.first().and_then(|f| f.date) {
        assert_eq!(
            first_forecast_date, today_local,
            "First forecast date should be Dec 28 (today in EST), got {}",
            first_forecast_date
        );
    }

    // CRITICAL: Verify context output fields
    let context = &context_builder.context;
    println!("\n=== Context Output Verification ===");
    println!(
        "Day 2 (tomorrow): {} - Max: {}",
        context.day2_name, context.day2_maxtemp
    );
    println!(
        "Sunrise: {} | Sunset: {}",
        context.sunrise_time, context.sunset_time
    );

    // With past_days=1, tomorrow (Dec 29) should be properly populated
    assert_eq!(
        context.day2_name, "Mon",
        "Tomorrow (Dec 29) should be Monday"
    );

    // Today's astronomical data should be present
    assert_ne!(
        context.sunrise_time, "NA",
        "Today's (Dec 28) sunrise should be populated with past_days=1"
    );
    assert_ne!(
        context.sunset_time, "NA",
        "Today's (Dec 28) sunset should be populated with past_days=1"
    );

    // Tomorrow's temps should be populated
    assert_ne!(
        context.day2_maxtemp, "NA",
        "Tomorrow's max temp should be populated"
    );

    // Cleanup
    unsafe {
        match original_tz {
            Some(tz) => std::env::set_var("TZ", tz),
            None => std::env::remove_var("TZ"),
        }
    }
}

/// Test that Open-Meteo dates convert correctly for Melbourne timezone (UTC+11)
///
/// **Issue**: When current_time is early UTC (00:00-10:00), subtracting 11 hours
/// underflows to the previous day, but the date wasn't being decremented.
///
/// **Clock**: 2025-10-26T00:00:00Z = 2025-10-26 11:00 AEDT (Melbourne)
#[test]
#[serial]
fn test_open_meteo_date_conversion_melbourne_midnight_utc() {
    // Save original TZ and set to Melbourne
    let original_tz = std::env::var("TZ").ok();
    unsafe {
        std::env::set_var("TZ", "Australia/Melbourne");
    }

    // Clock at 00:00 UTC = 11:00 AEDT (11AM Melbourne time)
    let clock = FixedClock::new(Utc.with_ymd_and_hms(2025, 10, 26, 0, 0, 0).unwrap());

    let today_local = clock.now_local().date_naive();
    println!("Today (local): {}", today_local);

    let daily_forecasts =
        load_open_meteo_daily_forecasts("tests/fixtures/open_meteo_forecast.json");

    let mut context_builder = ContextBuilder::new();
    context_builder.with_daily_forecast_data(daily_forecasts.clone(), &clock);

    println!("\nConverted forecast dates:");
    for (i, forecast) in daily_forecasts.iter().enumerate() {
        if let Some(date) = forecast.date {
            // date is already NaiveDate
            println!("  Forecast[{}]: Date={}", i, date);
        }
    }

    let dates_in_context: Vec<NaiveDate> = daily_forecasts.iter().filter_map(|f| f.date).collect();

    println!("\nLocal dates in context: {:?}", dates_in_context);

    // Verify that today (2025-10-26) is present in the forecasts
    assert!(
        dates_in_context.contains(&today_local),
        "Today's date ({}) should be present in forecasts. Found: {:?}",
        today_local,
        dates_in_context
    );

    // CRITICAL: Verify context output fields
    let context = &context_builder.context;
    println!("\n=== Context Output Verification ===");
    println!(
        "Day 2 (tomorrow): {} - Max: {}",
        context.day2_name, context.day2_maxtemp
    );
    println!(
        "Day 3: {} - Max: {}",
        context.day3_name, context.day3_maxtemp
    );
    println!(
        "Sunrise: {} | Sunset: {}",
        context.sunrise_time, context.sunset_time
    );

    // Verify day names are populated
    assert_ne!(
        context.day2_name, "NA",
        "Tomorrow's day name should be populated"
    );
    assert_ne!(
        context.day3_name, "NA",
        "Day 3's day name should be populated"
    );

    // Verify temperature fields are populated
    assert_ne!(
        context.day2_maxtemp, "NA",
        "Tomorrow's max temp should be populated"
    );

    // Verify sunrise/sunset are populated
    assert_ne!(
        context.sunrise_time, "NA",
        "Today's sunrise should be populated"
    );
    assert_ne!(
        context.sunset_time, "NA",
        "Today's sunset should be populated"
    );

    // Cleanup: restore original TZ
    unsafe {
        match original_tz {
            Some(tz) => std::env::set_var("TZ", &tz),
            None => std::env::remove_var("TZ"),
        }
    }
}

/// Test the boundary case: current_time that causes wrapping in both directions
///
/// This test covers multiple times that could cause wrapping issues:
/// - Midnight UTC boundaries
/// - Times that wrap to next/previous day when adjusted
/// - Both positive (NY) and negative (Melbourne) timezone offsets
#[test]
#[serial]
fn test_open_meteo_date_conversion_boundary_times() {
    // Test multiple times that could cause wrapping issues
    let test_cases = vec![
        ("2025-10-25T00:00:00Z", "America/New_York", "Midnight UTC"),
        (
            "2025-10-25T05:00:00Z",
            "America/New_York",
            "5AM UTC (midnight NY)",
        ),
        (
            "2025-10-25T19:00:00Z",
            "America/New_York",
            "7PM UTC (wraps to next day in calc)",
        ),
        (
            "2025-10-25T23:59:00Z",
            "America/New_York",
            "Just before midnight UTC",
        ),
        (
            "2025-10-26T00:00:00Z",
            "Australia/Melbourne",
            "Midnight UTC (11AM Melbourne)",
        ),
        (
            "2025-10-26T13:00:00Z",
            "Australia/Melbourne",
            "1PM UTC (midnight Melbourne)",
        ),
    ];

    for (time_str, tz, description) in test_cases {
        let original_tz = std::env::var("TZ").ok();
        unsafe {
            std::env::set_var("TZ", tz);
        }

        let clock = FixedClock::from_rfc3339(time_str).expect("Failed to create fixed clock");

        let today_local = clock.now_local().date_naive();
        let daily_forecasts =
            load_open_meteo_daily_forecasts("tests/fixtures/open_meteo_forecast.json");

        let mut context_builder = ContextBuilder::new();
        context_builder.with_daily_forecast_data(daily_forecasts.clone(), &clock);

        let dates_in_context: Vec<NaiveDate> =
            daily_forecasts.iter().filter_map(|f| f.date).collect();

        println!(
            "{}: Clock={} (TZ: {}) -> Local={} | Dates: {:?}",
            description, time_str, tz, today_local, dates_in_context
        );

        // Verify no panics and dates are reasonable
        assert!(
            !dates_in_context.is_empty(),
            "Should have some forecast dates for case: {}",
            description
        );

        // Verify dates are within a reasonable range (not wildly off)
        for date in &dates_in_context {
            let days_diff = date.signed_duration_since(today_local).num_days().abs();
            assert!(
                days_diff <= 14,
                "Date {} is too far from today {} ({} days) for case: {}",
                date,
                today_local,
                days_diff,
                description
            );
        }

        // CRITICAL: Verify context output fields are populated
        let context = &context_builder.context;
        println!(
            "  Context: day2_name={} day2_max={} sunrise={}",
            context.day2_name, context.day2_maxtemp, context.sunrise_time
        );

        // Verify basic context fields are populated (not empty strings)
        assert!(
            !context.day2_name.is_empty(),
            "Day 2 name should be populated for case: {}",
            description
        );
        assert!(
            !context.day2_maxtemp.is_empty(),
            "Day 2 max temp should be populated for case: {}",
            description
        );
        assert!(
            !context.sunrise_time.is_empty(),
            "Sunrise time should be populated for case: {}",
            description
        );

        // Cleanup: restore original TZ
        unsafe {
            match original_tz {
                Some(ref tz_val) => std::env::set_var("TZ", tz_val),
                None => std::env::remove_var("TZ"),
            }
        }
    }
}
