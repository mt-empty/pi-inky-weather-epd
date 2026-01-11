/// Tests for daylight saving time (DST) handling in API data conversions
///
/// These tests verify that weather API responses (BOM and Open-Meteo) are correctly
/// converted to domain models with proper timezone handling during DST transitions.
///
/// Australia (Melbourne/Sydney) DST:
/// - Starts: First Sunday in October at 2:00 AM → 3:00 AM (AEST → AEDT, UTC+10 → UTC+11)
/// - Ends: First Sunday in April at 3:00 AM → 2:00 AM (AEDT → AEST, UTC+11 → UTC+10)
use chrono::{Datelike, Local, Timelike};
use serial_test::serial;
use std::env;

/// Test BOM API data around DST spring forward transition
/// BOM returns UTC times - verify the 1-hour gap (2 AM doesn't exist)
/// Spring: First Sunday in October, 2:00 AM → 3:00 AM (AEST → AEDT)
#[test]
#[serial]
fn test_bom_forecast_time_conversion_during_dst() {
    use pi_inky_weather_epd::apis::bom::models::HourlyForecast as BomHourlyForecast;
    use pi_inky_weather_epd::domain::models::HourlyForecast as DomainHourlyForecast;

    // Set timezone to Australia/Melbourne for consistent test behavior
    let original_tz = env::var("TZ").ok();
    env::set_var("TZ", "Australia/Melbourne");

    // Test multiple hours around the DST transition on Oct 5, 2025
    let test_cases = vec![
        // Before DST: 1:00 AM AEST (Oct 5)
        ("2025-10-04T15:00:00Z", 1, 5, "Before DST"),
        // After DST: 3:00 AM AEDT (Oct 5) - 2 AM is skipped!
        ("2025-10-04T16:00:00Z", 3, 5, "After DST - skipped 2 AM"),
        // After DST: 4:00 AM AEDT (Oct 5)
        ("2025-10-04T17:00:00Z", 4, 5, "After DST"),
    ];

    for (utc_time, expected_hour, expected_day, description) in test_cases {
        let json = format!(
            r#"{{
                "rain": {{"amount": {{"min": null, "max": null, "units": "mm"}}, "chance": 10}},
                "temp": 18,
                "temp_feels_like": 16,
                "wind": {{
                    "speed_knot": 8,
                    "speed_kilometre": 15,
                    "direction": "N",
                    "gust_speed_knot": 12,
                    "gust_speed_kilometre": 22
                }},
                "relative_humidity": 65,
                "uv": 5,
                "time": "{}",
                "is_night": false
            }}"#,
            utc_time
        );

        let bom_forecast: BomHourlyForecast = serde_json::from_str(&json).unwrap();
        let domain_forecast: DomainHourlyForecast = bom_forecast.into();

        // Convert to local time for display
        let local_time = domain_forecast.time.with_timezone(&Local);

        assert_eq!(
            local_time.hour(),
            expected_hour,
            "{}: Expected hour {} but got {}",
            description,
            expected_hour,
            local_time.hour()
        );
        assert_eq!(
            local_time.day(),
            expected_day,
            "{}: Expected day {} but got {}",
            description,
            expected_day,
            local_time.day()
        );
    }

    // Verify the gap: hour sequence should be 1, 3, 4 (2 is skipped)
    // This demonstrates the spring forward behavior

    // Restore original timezone
    match original_tz {
        Some(tz) => env::set_var("TZ", tz),
        None => env::remove_var("TZ"),
    }
}

/// Test Open-Meteo API data during fall back DST transition
/// Open-Meteo returns UTC times (timezone=UTC) - verify duplicate local hour (2 AM happens twice)
/// Fall: First Sunday in April, 3:00 AM → 2:00 AM (AEDT → AEST, UTC+11 → UTC+10)
#[test]
#[serial]
fn test_open_meteo_forecast_time_conversion_during_dst() {
    use pi_inky_weather_epd::apis::open_meteo::models::OpenMeteoHourlyResponse;

    // Set timezone to Australia/Melbourne for consistent test behavior
    let original_tz = env::var("TZ").ok();
    env::set_var("TZ", "Australia/Melbourne");

    // Test multiple hours around the DST fall back transition on April 6, 2025
    // Open-Meteo returns UTC times (timezone=UTC parameter)
    // UTC times: 14:00, 15:00, 16:00, 17:00 on April 5
    // Local times: 1 AM AEDT, 2 AM AEDT, 2 AM AEST (duplicate!), 3 AM AEST on April 6
    let json = r#"{
        "latitude": -37.75,
        "longitude": 144.875,
        "timezone": "UTC",
        "timezone_abbreviation": "UTC",
        "current_units": {"time": "iso8601", "interval": "seconds", "is_day": ""},
        "current": {"time": "2025-04-05T14:00", "interval": 900, "is_day": 1},
        "hourly_units": {
            "time": "iso8601",
            "temperature_2m": "°C",
            "apparent_temperature": "°C",
            "precipitation_probability": "%",
            "precipitation": "mm",
            "snowfall": "cm",
            "uv_index": "",
            "wind_speed_10m": "km/h",
            "wind_gusts_10m": "km/h",
            "relative_humidity_2m": "%"
        },
        "hourly": {
            "time": [
                "2025-04-05T14:00",
                "2025-04-05T15:00",
                "2025-04-05T16:00",
                "2025-04-05T17:00"
            ],
            "temperature_2m": [19.5, 18.5, 17.8, 17.0],
            "apparent_temperature": [18.2, 17.2, 16.5, 15.8],
            "precipitation_probability": [10, 20, 15, 10],
            "precipitation": [0.0, 0.0, 0.0, 0.0],
            "snowfall": [0.0, 0.0, 0.0, 0.0],
            "uv_index": [1, 2, 3, 4],
            "wind_speed_10m": [10, 15, 12, 10],
            "wind_gusts_10m": [18, 22, 18, 15],
            "relative_humidity_2m": [80, 75, 78, 80],
            "cloud_cover": [40, 35, 28, 25]
        },
        "daily_units": {
            "time": "iso8601",
            "sunrise": "iso8601",
            "sunset": "iso8601",
            "temperature_2m_max": "°C",
            "temperature_2m_min": "°C",
            "precipitation_sum": "mm",
            "precipitation_probability_max": "%",
            "snowfall_sum": "cm"
        },
        "daily": {
            "time": ["2025-04-06"],
            "sunrise": ["2025-04-06T07:15"],
            "sunset": ["2025-04-06T18:30"],
            "temperature_2m_max": [22.5],
            "temperature_2m_min": [15.2],
            "precipitation_sum": [0.0],
            "precipitation_probability_max": [20],
            "snowfall_sum": [0.0],
            "cloud_cover_mean": [32]
        }
    }"#;

    let response: OpenMeteoHourlyResponse = serde_json::from_str(json).unwrap();
    let domain_forecasts: Vec<pi_inky_weather_epd::domain::models::HourlyForecast> =
        response.into();

    // Verify we get 4 hourly forecasts
    assert_eq!(domain_forecasts.len(), 4, "Should have 4 hourly forecasts");

    // Test each forecast hour when converted to local time
    let test_cases = vec![
        (0, 1, 6, "Before fall back: 1 AM AEDT"),
        (1, 2, 6, "Before fall back: 2 AM AEDT"),
        (2, 2, 6, "After fall back: 2 AM AEST (duplicate hour!)"),
        (3, 3, 6, "After fall back: 3 AM AEST"),
    ];

    for (index, expected_hour, expected_day, description) in test_cases {
        let forecast = &domain_forecasts[index];
        let local_time = forecast.time.with_timezone(&Local);

        assert_eq!(
            local_time.hour(),
            expected_hour,
            "{}: Expected hour {} but got {}",
            description,
            expected_hour,
            local_time.hour()
        );
        assert_eq!(
            local_time.day(),
            expected_day,
            "{}: Expected day {} but got {}",
            description,
            expected_day,
            local_time.day()
        );
    }

    // Verify the duplicate hour: sequence should be 1, 2, 2, 3 (2 AM happens twice)
    // This demonstrates the fall back behaviour
    let local_hours: Vec<u32> = domain_forecasts
        .iter()
        .map(|f| f.time.with_timezone(&Local).hour())
        .collect();
    assert_eq!(
        local_hours,
        vec![1, 2, 2, 3],
        "Expected hours 1, 2, 2, 3 showing duplicate 2 AM during fall back"
    );

    // Restore original timezone
    match original_tz {
        Some(tz) => env::set_var("TZ", tz),
        None => env::remove_var("TZ"),
    }
}
