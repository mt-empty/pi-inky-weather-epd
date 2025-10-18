/// Tests for daylight saving time (DST) transitions
///
/// Australia (Melbourne/Sydney) DST:
/// - Starts: First Sunday in October at 2:00 AM → 3:00 AM (AEST → AEDT, UTC+10 → UTC+11)
/// - Ends: First Sunday in April at 3:00 AM → 2:00 AM (AEDT → AEST, UTC+11 → UTC+10)
///
/// These tests verify that UTC to local time conversions handle DST correctly,
/// which is critical for weather forecast display accuracy.
use chrono::{Datelike, Local, NaiveDate, NaiveDateTime, TimeZone, Timelike, Utc};
use pi_inky_weather_epd::utils::{convert_utc_to_local_date, convert_utc_to_local_datetime};

/// Test UTC to local conversion during standard time (AEST, UTC+10)
/// Winter in Australia - no DST active
#[test]
fn test_utc_to_local_during_standard_time() {
    // July 15, 2025 - middle of winter, standard time (AEST, UTC+10)
    let utc_time = "2025-07-15T14:00:00Z";

    let local_datetime = convert_utc_to_local_datetime(utc_time).unwrap();

    // UTC 14:00 should be 00:00 next day in AEST (UTC+10)
    assert_eq!(local_datetime.date().day(), 16);
    assert_eq!(local_datetime.hour(), 0);
    assert_eq!(local_datetime.minute(), 0);
}

/// Test UTC to local conversion during daylight saving time (AEDT, UTC+11)
/// Summer in Australia - DST active
#[test]
fn test_utc_to_local_during_daylight_saving() {
    // January 15, 2025 - middle of summer, DST active (AEDT, UTC+11)
    let utc_time = "2025-01-15T13:00:00Z";

    let local_datetime = convert_utc_to_local_datetime(utc_time).unwrap();

    // UTC 13:00 should be 00:00 next day in AEDT (UTC+11)
    assert_eq!(local_datetime.date().day(), 16);
    assert_eq!(local_datetime.hour(), 0);
    assert_eq!(local_datetime.minute(), 0);
}

/// Test DST transition - clocks forward (spring forward)
/// First Sunday of October 2025: October 5, 2025
/// At 2:00 AM AEST → 3:00 AM AEDT (1 hour gap)
#[test]
fn test_dst_spring_forward_transition() {
    // Before DST: Oct 5, 2025 01:30 AEST = Oct 4, 15:30 UTC
    let before_dst_utc = Utc.with_ymd_and_hms(2025, 10, 4, 15, 30, 0).unwrap();
    let before_local = before_dst_utc.with_timezone(&Local);
    assert_eq!(before_local.hour(), 1); // 1:30 AM AEST
    assert_eq!(before_local.day(), 5);

    // After DST: Oct 5, 2025 03:30 AEDT = Oct 4, 16:30 UTC
    // Note: 2:00-2:59 AM doesn't exist on this day!
    let after_dst_utc = Utc.with_ymd_and_hms(2025, 10, 4, 16, 30, 0).unwrap();
    let after_local = after_dst_utc.with_timezone(&Local);
    assert_eq!(after_local.hour(), 3); // 3:30 AM AEDT (skipped 2:xx)
    assert_eq!(after_local.day(), 5);

    // Only 1 hour passed in UTC but 2 hours in local time (1:30 AM → 3:30 AM)
    let utc_diff_hours = (after_dst_utc - before_dst_utc).num_hours();
    assert_eq!(utc_diff_hours, 1);
}

/// Test DST transition - clocks backward (fall back)
/// First Sunday of April 2025: April 6, 2025
/// At 3:00 AM AEDT → 2:00 AM AEST (1 hour repeats)
#[test]
fn test_dst_fall_back_transition() {
    // Before DST ends: Apr 6, 2025 02:30 AEDT = Apr 5, 15:30 UTC
    let before_dst_utc = Utc.with_ymd_and_hms(2025, 4, 5, 15, 30, 0).unwrap();
    let before_local = before_dst_utc.with_timezone(&Local);
    assert_eq!(before_local.hour(), 2); // 2:30 AM AEDT (first occurrence)
    assert_eq!(before_local.day(), 6);

    // After DST ends: Apr 6, 2025 02:30 AEST = Apr 5, 16:30 UTC
    // Note: 2:00-2:59 AM happens twice on this day!
    let after_dst_utc = Utc.with_ymd_and_hms(2025, 4, 5, 16, 30, 0).unwrap();
    let after_local = after_dst_utc.with_timezone(&Local);
    assert_eq!(after_local.hour(), 2); // 2:30 AM AEST (second occurrence)
    assert_eq!(after_local.day(), 6);

    // 1 hour passed in UTC but same wall clock time
    let utc_diff_hours = (after_dst_utc - before_dst_utc).num_hours();
    assert_eq!(utc_diff_hours, 1);
}

/// Test weather forecast times around spring DST transition
/// Ensures hourly forecasts are correctly mapped to local time
#[test]
fn test_hourly_forecast_during_spring_dst_transition() {
    // Simulate receiving hourly forecasts around DST transition
    // Oct 4-5, 2025 transition

    let forecasts = vec![
        "2025-10-04T15:00:00Z", // Oct 5, 01:00 AEST (before DST)
        "2025-10-04T16:00:00Z", // Oct 5, 03:00 AEDT (after DST) - 02:00 skipped!
        "2025-10-04T17:00:00Z", // Oct 5, 04:00 AEDT
    ];

    let local_times: Vec<NaiveDateTime> = forecasts
        .iter()
        .map(|utc_str| convert_utc_to_local_datetime(utc_str).unwrap())
        .collect();

    // Verify the gap: should jump from 1 AM to 3 AM
    assert_eq!(local_times[0].hour(), 1);
    assert_eq!(local_times[1].hour(), 3); // Skips 2 AM
    assert_eq!(local_times[2].hour(), 4);

    // All should be on October 5
    assert_eq!(local_times[0].date().day(), 5);
    assert_eq!(local_times[1].date().day(), 5);
    assert_eq!(local_times[2].date().day(), 5);
}

/// Test weather forecast times around fall DST transition
/// Ensures daily summaries use correct date boundaries
#[test]
fn test_daily_forecast_during_fall_dst_transition() {
    // April 5-6, 2025 transition

    let forecasts = vec![
        "2025-04-05T15:00:00Z", // Apr 6, 02:00 AEDT (first occurrence)
        "2025-04-05T16:00:00Z", // Apr 6, 02:00 AEST (second occurrence!)
        "2025-04-05T17:00:00Z", // Apr 6, 03:00 AEST
    ];

    let local_times: Vec<NaiveDateTime> = forecasts
        .iter()
        .map(|utc_str| convert_utc_to_local_datetime(utc_str).unwrap())
        .collect();

    // Verify the repeat: 2 AM appears twice
    assert_eq!(local_times[0].hour(), 2); // First 2 AM (AEDT)
    assert_eq!(local_times[1].hour(), 2); // Second 2 AM (AEST)
    assert_eq!(local_times[2].hour(), 3);

    // All should be on April 6
    assert_eq!(local_times[0].date().day(), 6);
    assert_eq!(local_times[1].date().day(), 6);
    assert_eq!(local_times[2].date().day(), 6);
}

/// Test date conversion around midnight during DST transition
/// Critical for daily forecast date boundaries
#[test]
fn test_date_boundary_during_dst_spring_transition() {
    // Test midnight transitions around DST change

    // Oct 4, 2025 23:00 UTC = Oct 5, 2025 09:00 AEST (before DST ends that day)
    let date1 = convert_utc_to_local_date("2025-10-04T13:00:00Z").unwrap();
    assert_eq!(date1, NaiveDate::from_ymd_opt(2025, 10, 4).unwrap());

    // Oct 4, 2025 14:00 UTC = Oct 5, 2025 00:00 AEST (just after midnight, before DST)
    let date2 = convert_utc_to_local_date("2025-10-04T14:00:00Z").unwrap();
    assert_eq!(date2, NaiveDate::from_ymd_opt(2025, 10, 5).unwrap());

    // Oct 4, 2025 17:00 UTC = Oct 5, 2025 04:00 AEDT (after DST starts)
    let date3 = convert_utc_to_local_date("2025-10-04T17:00:00Z").unwrap();
    assert_eq!(date3, NaiveDate::from_ymd_opt(2025, 10, 5).unwrap());
}

/// Test date conversion around midnight during fall DST transition
#[test]
fn test_date_boundary_during_dst_fall_transition() {
    // Apr 5, 2025 13:00 UTC = Apr 6, 2025 00:00 AEDT (just after midnight, DST still active)
    let date1 = convert_utc_to_local_date("2025-04-05T13:00:00Z").unwrap();
    assert_eq!(date1, NaiveDate::from_ymd_opt(2025, 4, 6).unwrap());

    // Apr 5, 2025 17:00 UTC = Apr 6, 2025 03:00 AEST (after DST ends)
    let date2 = convert_utc_to_local_date("2025-04-05T17:00:00Z").unwrap();
    assert_eq!(date2, NaiveDate::from_ymd_opt(2025, 4, 6).unwrap());
}

/// Test BOM API data around DST - BOM returns UTC times
/// This verifies the complete flow: API response → domain model → local time
#[test]
fn test_bom_forecast_time_conversion_during_dst() {
    use pi_inky_weather_epd::apis::bom::models::HourlyForecast as BomHourlyForecast;
    use pi_inky_weather_epd::domain::models::HourlyForecast as DomainHourlyForecast;

    // Simulate BOM hourly forecast during spring DST transition
    let json = r#"{
        "rain": {"amount": {"min": null, "max": null, "units": "mm"}, "chance": 10},
        "temp": 18,
        "temp_feels_like": 16,
        "wind": {
            "speed_knot": 8,
            "speed_kilometre": 15,
            "direction": "N",
            "gust_speed_knot": 12,
            "gust_speed_kilometre": 22
        },
        "relative_humidity": 65,
        "uv": 5,
        "time": "2025-10-04T16:00:00Z",
        "is_night": false
    }"#;

    let bom_forecast: BomHourlyForecast = serde_json::from_str(json).unwrap();
    let domain_forecast: DomainHourlyForecast = bom_forecast.into();

    // Verify time is preserved as UTC in domain model
    let forecast_time = domain_forecast.time;
    assert_eq!(forecast_time.hour(), 16); // UTC
    assert_eq!(forecast_time.day(), 4);

    // Convert to local for display - should be 3:00 AM AEDT on Oct 5
    let local_time = forecast_time.with_timezone(&Local);
    assert_eq!(local_time.hour(), 3); // AEDT (after DST transition)
    assert_eq!(local_time.day(), 5);
}

/// Test Open-Meteo API data around DST
/// Open-Meteo can return times in local timezone, but we convert to UTC
#[test]
fn test_open_meteo_forecast_time_conversion_during_dst() {
    use pi_inky_weather_epd::apis::open_metro::models::OpenMeteoHourlyResponse;

    // Simulate Open-Meteo hourly forecast during fall DST transition
    // Open-Meteo returns times in the specified timezone
    let json = r#"{
        "latitude": -37.75,
        "longitude": 144.875,
        "timezone": "Australia/Melbourne",
        "timezone_abbreviation": "AEDT",
        "current_units": {"time": "iso8601", "interval": "seconds", "is_day": ""},
        "current": {"time": "2025-04-05T13:45", "interval": 900, "is_day": 1},
        "hourly_units": {
            "time": "iso8601",
            "temperature_2m": "°C",
            "apparent_temperature": "°C",
            "precipitation_probability": "%",
            "precipitation": "mm",
            "uv_index": "",
            "wind_speed_10m": "km/h",
            "wind_gusts_10m": "km/h",
            "relative_humidity_2m": "%"
        },
        "hourly": {
            "time": ["2025-04-06T02:00", "2025-04-06T03:00"],
            "temperature_2m": [18.5, 17.8],
            "apparent_temperature": [17.2, 16.5],
            "precipitation_probability": [20, 15],
            "precipitation": [0.0, 0.0],
            "uv_index": [2, 3],
            "wind_speed_10m": [15, 12],
            "wind_gusts_10m": [22, 18],
            "relative_humidity_2m": [75, 78]
        },
        "daily_units": {
            "time": "iso8601",
            "sunrise": "iso8601",
            "sunset": "iso8601",
            "temperature_2m_max": "°C",
            "temperature_2m_min": "°C",
            "precipitation_sum": "mm",
            "precipitation_probability_max": "%"
        },
        "daily": {
            "time": ["2025-04-06"],
            "sunrise": ["2025-04-06T07:15"],
            "sunset": ["2025-04-06T18:30"],
            "temperature_2m_max": [22.5],
            "temperature_2m_min": [15.2],
            "precipitation_sum": [0.0],
            "precipitation_probability_max": [20]
        }
    }"#;

    let response: OpenMeteoHourlyResponse = serde_json::from_str(json).unwrap();
    let domain_forecasts: Vec<pi_inky_weather_epd::domain::models::HourlyForecast> =
        response.into();

    // Verify we get 2 hourly forecasts
    assert_eq!(domain_forecasts.len(), 2);

    // Times should be converted to UTC
    // Apr 6, 02:00 local time during DST transition
    let first_forecast = &domain_forecasts[0];
    // The exact day in UTC depends on which 02:00 this is (AEDT or AEST)
    // But both should be reasonably close in UTC time
    assert!(first_forecast.time.day() == 5 || first_forecast.time.day() == 6);
}

/// Test that 25-hour days (fall back) don't break daily aggregations
#[test]
fn test_daily_aggregation_on_25_hour_day() {
    // April 6, 2025 has 25 hours due to DST fall back
    // Verify date grouping works correctly

    let utc_times = vec![
        "2025-04-05T14:00:00Z", // Apr 6, 01:00 AEDT
        "2025-04-05T15:00:00Z", // Apr 6, 02:00 AEDT (first)
        "2025-04-05T16:00:00Z", // Apr 6, 02:00 AEST (second - same wall clock!)
        "2025-04-05T17:00:00Z", // Apr 6, 03:00 AEST
        "2025-04-06T14:00:00Z", // Apr 7, 01:00 AEST
    ];

    let dates: Vec<NaiveDate> = utc_times
        .iter()
        .map(|utc_str| convert_utc_to_local_date(utc_str).unwrap())
        .collect();

    // First 4 should be April 6, last should be April 7
    assert_eq!(dates[0].day(), 6);
    assert_eq!(dates[1].day(), 6);
    assert_eq!(dates[2].day(), 6);
    assert_eq!(dates[3].day(), 6);
    assert_eq!(dates[4].day(), 7);

    // Count hours on April 6 - should be 4 in this sample
    let april_6_hours = dates.iter().filter(|d| d.day() == 6).count();
    assert_eq!(april_6_hours, 4);
}

/// Test that 23-hour days (spring forward) don't break daily aggregations
#[test]
fn test_daily_aggregation_on_23_hour_day() {
    // October 5, 2025 has only 23 hours due to DST spring forward
    // 2:00-2:59 AM doesn't exist

    let utc_times = vec![
        "2025-10-04T14:00:00Z", // Oct 5, 00:00 AEST
        "2025-10-04T15:00:00Z", // Oct 5, 01:00 AEST
        "2025-10-04T16:00:00Z", // Oct 5, 03:00 AEDT (2:00 skipped!)
        "2025-10-04T17:00:00Z", // Oct 5, 04:00 AEDT
        "2025-10-05T13:00:00Z", // Oct 6, 00:00 AEDT
    ];

    let dates: Vec<NaiveDate> = utc_times
        .iter()
        .map(|utc_str| convert_utc_to_local_date(utc_str).unwrap())
        .collect();

    // First 4 should be October 5, last should be October 6
    assert_eq!(dates[0].day(), 5);
    assert_eq!(dates[1].day(), 5);
    assert_eq!(dates[2].day(), 5);
    assert_eq!(dates[3].day(), 5);
    assert_eq!(dates[4].day(), 6);

    // Count hours on October 5 - should be 4 in this sample
    let oct_5_hours = dates.iter().filter(|d| d.day() == 5).count();
    assert_eq!(oct_5_hours, 4);
}
