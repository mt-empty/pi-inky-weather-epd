/// Test ContextBuilder::with_daily_forecast_data for New York timezone
///
/// Verifies that daily forecast data with noon UTC timestamps correctly maps
/// to local dates without shifting to the previous day in EST (UTC-5)
use chrono::{NaiveDate, NaiveDateTime, TimeZone, Utc};
use pi_inky_weather_epd::{
    clock::FixedClock,
    configs::settings::TemperatureUnit,
    dashboard::context::ContextBuilder,
    domain::models::{Astronomical, DailyForecast, Precipitation, Temperature},
};
use serial_test::serial;

/// Helper to create a Temperature in Celsius
fn temp_c(value: f32) -> Temperature {
    Temperature::new(value, TemperatureUnit::C)
}

/// Test with_daily_forecast_data in New York timezone (EST, UTC-5)
/// Current time: Dec 17, 2025 at 10:00 AM EST
#[test]
#[serial]
fn test_with_daily_forecast_data_new_york_est() {
    // Set timezone to New York
    let original_tz = std::env::var("TZ").ok();
    unsafe { std::env::set_var("TZ", "America/New_York") };

    // Fixed clock: Dec 17, 2025 at 10:00 AM EST (15:00 UTC)
    let clock = FixedClock::new(Utc.with_ymd_and_hms(2025, 12, 17, 15, 0, 0).unwrap());

    // Create daily forecast data with noon UTC timestamps (the fix we implemented)
    // Dec 17-23, 2025: Wed, Thu, Fri, Sat, Sun, Mon, Tue
    let daily_forecasts = vec![
        DailyForecast {
            date: Some(NaiveDate::from_ymd_opt(2025, 12, 17).unwrap()),
            temp_max: Some(temp_c(9.9)),
            temp_min: Some(temp_c(-2.8)),
            precipitation: Some(Precipitation::new(Some(10), None, Some(0))),
            astronomical: Some(Astronomical {
                // Sunrise/sunset as NaiveDateTime (local wall-clock time)
                // 12:19 UTC = 7:19 AM EST, 21:33 UTC = 4:33 PM EST
                sunrise_time: Some(
                    NaiveDateTime::parse_from_str("2025-12-17 07:19:00", "%Y-%m-%d %H:%M:%S")
                        .unwrap(),
                ),
                sunset_time: Some(
                    NaiveDateTime::parse_from_str("2025-12-17 16:33:00", "%Y-%m-%d %H:%M:%S")
                        .unwrap(),
                ),
            }),
            cloud_cover: None,
            weather_code: None,
        },
        DailyForecast {
            date: Some(NaiveDate::from_ymd_opt(2025, 12, 18).unwrap()),
            temp_max: Some(temp_c(10.3)),
            temp_min: Some(temp_c(-1.2)),
            precipitation: Some(Precipitation::new(Some(30), None, Some(1))),
            astronomical: None,
            cloud_cover: None,
            weather_code: None,
        },
        DailyForecast {
            date: Some(NaiveDate::from_ymd_opt(2025, 12, 19).unwrap()),
            temp_max: Some(temp_c(11.5)),
            temp_min: Some(temp_c(1.9)),
            precipitation: Some(Precipitation::new(Some(50), None, Some(2))),
            astronomical: None,
            cloud_cover: None,
            weather_code: None,
        },
        DailyForecast {
            date: Some(NaiveDate::from_ymd_opt(2025, 12, 20).unwrap()),
            temp_max: Some(temp_c(2.2)),
            temp_min: Some(temp_c(-1.1)),
            precipitation: None,
            astronomical: None,
            cloud_cover: None,
            weather_code: None,
        },
        DailyForecast {
            date: Some(NaiveDate::from_ymd_opt(2025, 12, 21).unwrap()),
            temp_max: Some(temp_c(7.2)),
            temp_min: Some(temp_c(1.7)),
            precipitation: None,
            astronomical: None,
            cloud_cover: None,
            weather_code: None,
        },
        DailyForecast {
            date: Some(NaiveDate::from_ymd_opt(2025, 12, 22).unwrap()),
            temp_max: Some(temp_c(5.0)),
            temp_min: Some(temp_c(-1.5)),
            precipitation: None,
            astronomical: None,
            cloud_cover: None,
            weather_code: None,
        },
        DailyForecast {
            date: Some(NaiveDate::from_ymd_opt(2025, 12, 23).unwrap()),
            temp_max: Some(temp_c(1.3)),
            temp_min: Some(temp_c(-3.0)),
            precipitation: None,
            astronomical: None,
            cloud_cover: None,
            weather_code: None,
        },
    ];

    let mut builder = ContextBuilder::new();
    builder.with_daily_forecast_data(daily_forecasts, &clock);

    let context = builder.context;

    // Verify day names (Dec 18-23: Thu, Fri, Sat, Sun, Mon, Tue)
    assert_eq!(context.day2_name, "Thu", "Day 2 should be Thursday");
    assert_eq!(context.day3_name, "Fri", "Day 3 should be Friday");
    assert_eq!(context.day4_name, "Sat", "Day 4 should be Saturday");
    assert_eq!(context.day5_name, "Sun", "Day 5 should be Sunday");
    assert_eq!(context.day6_name, "Mon", "Day 6 should be Monday");
    assert_eq!(context.day7_name, "Tue", "Day 7 should be Tuesday");

    // Verify temperatures are correctly rounded and assigned
    // Day 0 (today, Dec 17) - only sunrise/sunset used
    assert_eq!(context.sunrise_time, "07:19", "Sunrise time incorrect");
    assert_eq!(context.sunset_time, "16:33", "Sunset time incorrect");

    // Day 2 (Thu, Dec 18): 10.3°C → 10, -1.2°C → -1
    assert_eq!(context.day2_maxtemp, "10", "Day 2 max temp incorrect");
    assert_eq!(context.day2_mintemp, "-1", "Day 2 min temp incorrect");

    // Day 3 (Fri, Dec 19): 11.5°C → 12, 1.9°C → 2
    assert_eq!(context.day3_maxtemp, "12", "Day 3 max temp incorrect");
    assert_eq!(context.day3_mintemp, "2", "Day 3 min temp incorrect");

    // Day 4 (Sat, Dec 20): 2.2°C → 2, -1.1°C → -1
    assert_eq!(context.day4_maxtemp, "2", "Day 4 max temp incorrect");
    assert_eq!(context.day4_mintemp, "-1", "Day 4 min temp incorrect");

    // Day 5 (Sun, Dec 21): 7.2°C → 7, 1.7°C → 2
    assert_eq!(context.day5_maxtemp, "7", "Day 5 max temp incorrect");
    assert_eq!(context.day5_mintemp, "2", "Day 5 min temp incorrect");

    // Day 6 (Mon, Dec 22): 5.0°C → 5, -1.5°C → -2
    assert_eq!(context.day6_maxtemp, "5", "Day 6 max temp incorrect");
    assert_eq!(context.day6_mintemp, "-2", "Day 6 min temp incorrect");

    // Day 7 (Tue, Dec 23): 1.3°C → 1, -3.0°C → -3
    assert_eq!(context.day7_maxtemp, "1", "Day 7 max temp incorrect");
    assert_eq!(context.day7_mintemp, "-3", "Day 7 min temp incorrect");

    // Restore original TZ
    unsafe {
        match original_tz {
            Some(tz) => std::env::set_var("TZ", tz),
            None => std::env::remove_var("TZ"),
        }
    }
}

/// Test that noon UTC timestamps don't cause date shifts in EST
#[test]
#[serial]
fn test_noon_utc_prevents_date_shift_in_est() {
    let original_tz = std::env::var("TZ").ok();
    unsafe { std::env::set_var("TZ", "America/New_York") };

    // Fixed clock: Dec 17, 2025 at 2:00 AM EST (early morning)
    let clock = FixedClock::new(Utc.with_ymd_and_hms(2025, 12, 17, 7, 0, 0).unwrap());

    // Test the critical case: noon UTC on Dec 17
    // 2025-12-17T12:00:00Z → 2025-12-17T07:00:00-05:00 (same day!)
    let daily_forecasts = vec![
        DailyForecast {
            date: Some(NaiveDate::from_ymd_opt(2025, 12, 17).unwrap()),
            temp_max: Some(temp_c(10.0)),
            temp_min: Some(temp_c(-3.0)),
            precipitation: None,
            astronomical: None,
            cloud_cover: None,
            weather_code: None,
        },
        DailyForecast {
            date: Some(NaiveDate::from_ymd_opt(2025, 12, 18).unwrap()),
            temp_max: Some(temp_c(11.0)),
            temp_min: Some(temp_c(-2.0)),
            precipitation: None,
            astronomical: None,
            cloud_cover: None,
            weather_code: None,
        },
        DailyForecast {
            date: Some(NaiveDate::from_ymd_opt(2025, 12, 19).unwrap()),
            temp_max: Some(temp_c(12.0)),
            temp_min: Some(temp_c(-1.0)),
            precipitation: None,
            astronomical: None,
            cloud_cover: None,
            weather_code: None,
        },
        DailyForecast {
            date: Some(NaiveDate::from_ymd_opt(2025, 12, 20).unwrap()),
            temp_max: Some(temp_c(13.0)),
            temp_min: Some(temp_c(0.0)),
            precipitation: None,
            astronomical: None,
            cloud_cover: None,
            weather_code: None,
        },
        DailyForecast {
            date: Some(NaiveDate::from_ymd_opt(2025, 12, 21).unwrap()),
            temp_max: Some(temp_c(14.0)),
            temp_min: Some(temp_c(1.0)),
            precipitation: None,
            astronomical: None,
            cloud_cover: None,
            weather_code: None,
        },
        DailyForecast {
            date: Some(NaiveDate::from_ymd_opt(2025, 12, 22).unwrap()),
            temp_max: Some(temp_c(15.0)),
            temp_min: Some(temp_c(2.0)),
            precipitation: None,
            astronomical: None,
            cloud_cover: None,
            weather_code: None,
        },
        DailyForecast {
            date: Some(NaiveDate::from_ymd_opt(2025, 12, 23).unwrap()),
            temp_max: Some(temp_c(16.0)),
            temp_min: Some(temp_c(3.0)),
            precipitation: None,
            astronomical: None,
            cloud_cover: None,
            weather_code: None,
        },
    ];

    let mut builder = ContextBuilder::new();
    builder.with_daily_forecast_data(daily_forecasts, &clock);

    let context = builder.context;

    // All days should be correctly assigned despite early morning test time
    assert_eq!(context.day2_name, "Thu");
    assert_eq!(context.day2_maxtemp, "11"); // Dec 18 data goes to day2
    assert_eq!(context.day3_maxtemp, "12"); // Dec 19 data goes to day3
    assert_eq!(context.day4_maxtemp, "13"); // Dec 20 data goes to day4
    assert_eq!(context.day5_maxtemp, "14"); // Dec 21 data goes to day5
    assert_eq!(context.day6_maxtemp, "15"); // Dec 22 data goes to day6
    assert_eq!(context.day7_maxtemp, "16"); // Dec 23 data goes to day7

    // Restore original TZ
    unsafe {
        match original_tz {
            Some(tz) => std::env::set_var("TZ", tz),
            None => std::env::remove_var("TZ"),
        }
    }
}

/// Test that dates before today are correctly skipped
#[test]
#[serial]
fn test_skips_past_dates() {
    let original_tz = std::env::var("TZ").ok();
    unsafe { std::env::set_var("TZ", "America/New_York") };

    // Fixed clock: Dec 19, 2025 at 10:00 AM EST
    let clock = FixedClock::new(Utc.with_ymd_and_hms(2025, 12, 19, 15, 0, 0).unwrap());

    // Include past dates (Dec 17, 18) which should be skipped
    let daily_forecasts = vec![
        DailyForecast {
            date: Some(NaiveDate::from_ymd_opt(2025, 12, 17).unwrap()),
            temp_max: Some(temp_c(10.0)),
            temp_min: Some(temp_c(-3.0)),
            precipitation: None,
            astronomical: None,
            cloud_cover: None,
            weather_code: None,
        },
        DailyForecast {
            date: Some(NaiveDate::from_ymd_opt(2025, 12, 18).unwrap()),
            temp_max: Some(temp_c(11.0)),
            temp_min: Some(temp_c(-2.0)),
            precipitation: None,
            astronomical: None,
            cloud_cover: None,
            weather_code: None,
        },
        DailyForecast {
            date: Some(NaiveDate::from_ymd_opt(2025, 12, 19).unwrap()),
            temp_max: Some(temp_c(12.0)),
            temp_min: Some(temp_c(-1.0)),
            precipitation: None,
            astronomical: None,
            cloud_cover: None,
            weather_code: None,
        },
        DailyForecast {
            date: Some(NaiveDate::from_ymd_opt(2025, 12, 20).unwrap()),
            temp_max: Some(temp_c(13.0)),
            temp_min: Some(temp_c(0.0)),
            precipitation: None,
            astronomical: None,
            cloud_cover: None,
            weather_code: None,
        },
        DailyForecast {
            date: Some(NaiveDate::from_ymd_opt(2025, 12, 21).unwrap()),
            temp_max: Some(temp_c(14.0)),
            temp_min: Some(temp_c(1.0)),
            precipitation: None,
            astronomical: None,
            cloud_cover: None,
            weather_code: None,
        },
        DailyForecast {
            date: Some(NaiveDate::from_ymd_opt(2025, 12, 22).unwrap()),
            temp_max: Some(temp_c(15.0)),
            temp_min: Some(temp_c(2.0)),
            precipitation: None,
            astronomical: None,
            cloud_cover: None,
            weather_code: None,
        },
        DailyForecast {
            date: Some(NaiveDate::from_ymd_opt(2025, 12, 23).unwrap()),
            temp_max: Some(temp_c(16.0)),
            temp_min: Some(temp_c(3.0)),
            precipitation: None,
            astronomical: None,
            cloud_cover: None,
            weather_code: None,
        },
        DailyForecast {
            date: Some(NaiveDate::from_ymd_opt(2025, 12, 24).unwrap()),
            temp_max: Some(temp_c(17.0)),
            temp_min: Some(temp_c(4.0)),
            precipitation: None,
            astronomical: None,
            cloud_cover: None,
            weather_code: None,
        },
        DailyForecast {
            date: Some(NaiveDate::from_ymd_opt(2025, 12, 25).unwrap()),
            temp_max: Some(temp_c(18.0)),
            temp_min: Some(temp_c(5.0)),
            precipitation: None,
            astronomical: None,
            cloud_cover: None,
            weather_code: None,
        },
    ];

    let mut builder = ContextBuilder::new();
    builder.with_daily_forecast_data(daily_forecasts, &clock);

    let context = builder.context;

    // Dec 19 is today (day 0), so day2 should be Dec 20 (Sat)
    assert_eq!(context.day2_name, "Sat");
    assert_eq!(context.day2_maxtemp, "13"); // Dec 20 → day2
    assert_eq!(context.day3_maxtemp, "14"); // Dec 21 → day3
    assert_eq!(context.day4_maxtemp, "15"); // Dec 22 → day4
    assert_eq!(context.day5_maxtemp, "16"); // Dec 23 → day5
    assert_eq!(context.day6_maxtemp, "17"); // Dec 24 → day6
    assert_eq!(context.day7_maxtemp, "18"); // Dec 25 → day7

    // Restore original TZ
    unsafe {
        match original_tz {
            Some(tz) => std::env::set_var("TZ", tz),
            None => std::env::remove_var("TZ"),
        }
    }
}
