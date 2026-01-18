//! Layer 1 Tests: Open-Meteo Provider JSON deserialization and conversion
//!
//! These tests verify:
//! 1. Open-Meteo JSON fixtures can be loaded and parsed
//! 2. The data in the files has expected structure
//!
//! Test fixtures are stored in tests/fixtures/ directory to avoid
//! dependency on runtime cache files from executing the binary.

use pi_inky_weather_epd::apis::open_meteo::models::{
    OpenMeteoDailyResponse, OpenMeteoHourlyResponse,
};
use std::fs;

/// Test that Open-Meteo hourly forecast fixture deserializes
#[test]
fn test_load_open_meteo_hourly_fixture() {
    let json = fs::read_to_string("tests/fixtures/open_meteo_hourly_forecast.json")
        .expect("Failed to read Open-Meteo hourly forecast fixture file");

    let result: Result<OpenMeteoHourlyResponse, _> = serde_json::from_str(&json);
    assert!(
        result.is_ok(),
        "Failed to deserialize Open-Meteo hourly forecast: {:?}",
        result.err()
    );

    let response = result.unwrap();

    // Validate hourly data
    assert!(
        !response.hourly.time.is_empty(),
        "Expected at least one hourly forecast entry"
    );
    assert_eq!(
        response.hourly.time.len(),
        response.hourly.temperature_2m.len(),
        "All hourly arrays should have same length"
    );
}

/// Test that Open-Meteo daily forecast fixture deserializes
#[test]
fn test_load_open_meteo_daily_fixture() {
    let json = fs::read_to_string("tests/fixtures/open_meteo_daily_forecast.json")
        .expect("Failed to read Open-Meteo daily forecast fixture file");

    let result: Result<OpenMeteoDailyResponse, _> = serde_json::from_str(&json);
    assert!(
        result.is_ok(),
        "Failed to deserialize Open-Meteo daily forecast: {:?}",
        result.err()
    );

    let response = result.unwrap();

    // Validate daily data
    assert!(
        !response.daily.time.is_empty(),
        "Expected at least one daily forecast entry"
    );
    assert_eq!(
        response.daily.time.len(),
        response.daily.temperature_2m_max.len(),
        "All daily arrays should have same length"
    );
}

/// Test snowfall data is captured in hourly fixtures
#[test]
fn test_open_meteo_hourly_includes_snowfall() {
    let json = fs::read_to_string("tests/fixtures/open_meteo_hourly_forecast.json")
        .expect("Failed to read Open-Meteo hourly forecast fixture file");

    let response: OpenMeteoHourlyResponse = serde_json::from_str(&json).unwrap();

    // Verify snowfall array exists and has correct length
    assert_eq!(
        response.hourly.snowfall.len(),
        response.hourly.time.len(),
        "Snowfall array should match time array length"
    );

    // Verify snowfall data contains expected values (some zeros, some non-zero)
    let has_snowfall = response.hourly.snowfall.iter().any(|&s| s > 0.0);
    assert!(
        has_snowfall,
        "Test fixture should contain at least some snowfall data"
    );
}

/// Test snowfall data is captured in daily fixtures
#[test]
fn test_open_meteo_daily_includes_snowfall_sum() {
    let json = fs::read_to_string("tests/fixtures/open_meteo_daily_forecast.json")
        .expect("Failed to read Open-Meteo daily forecast fixture file");

    let response: OpenMeteoDailyResponse = serde_json::from_str(&json).unwrap();

    // Verify snowfall_sum array exists and has correct length
    assert_eq!(
        response.daily.snowfall_sum.len(),
        response.daily.time.len(),
        "Snowfall sum array should match time array length"
    );

    // Verify daily snowfall contains expected values
    let has_snowfall = response.daily.snowfall_sum.iter().any(|&s| s > 0.0);
    assert!(
        has_snowfall,
        "Test fixture should contain at least some daily snowfall data"
    );
}

/// Test Open-Meteo hourly forecast has expected fields and ranges
#[test]
fn test_open_meteo_hourly_fields() {
    let json = fs::read_to_string("tests/fixtures/open_meteo_hourly_forecast.json")
        .expect("Failed to read Open-Meteo hourly forecast fixture file");

    let response: OpenMeteoHourlyResponse = serde_json::from_str(&json).unwrap();
    let hourly = &response.hourly;

    for i in 0..hourly.time.len() {
        // Verify temperature is in reasonable range
        let temp = hourly.temperature_2m[i];
        assert!(
            temp > -50.0 && temp < 60.0,
            "Temperature should be in reasonable range"
        );

        let apparent_temp = hourly.apparent_temperature[i];
        assert!(
            apparent_temp.is_finite(),
            "Apparent temperature should be finite"
        );

        // Verify precipitation probability
        let precip_prob = hourly.precipitation_probability[i];
        assert!(
            precip_prob <= 100,
            "Precipitation probability should be <= 100%"
        );

        // Verify precipitation amount
        let precip = hourly.precipitation[i];
        assert!(
            (0.0..500.0).contains(&precip),
            "Precipitation should be reasonable"
        );

        // Verify UV index
        let uv = hourly.uv_index[i];
        assert!((0.0..20.0).contains(&uv), "UV index should be < 20");

        // Verify wind speed
        let wind = hourly.wind_speed_10m[i];
        assert!(
            (0.0..500.0).contains(&wind),
            "Wind speed should be reasonable"
        );

        // Verify humidity
        let humidity = hourly.relative_humidity_2m[i];
        assert!(humidity <= 100, "Humidity should be <= 100%");
    }
}

/// Test Open-Meteo daily forecast has expected fields and ranges
#[test]
fn test_open_meteo_daily_fields() {
    let json = fs::read_to_string("tests/fixtures/open_meteo_daily_forecast.json")
        .expect("Failed to read Open-Meteo daily forecast fixture file");

    let response: OpenMeteoDailyResponse = serde_json::from_str(&json).unwrap();
    let daily = &response.daily;

    for i in 0..daily.time.len() {
        // Verify temperature max/min
        let temp_max = daily.temperature_2m_max[i];
        assert!(
            temp_max > -50.0 && temp_max < 60.0,
            "Max temperature should be in reasonable range"
        );

        let temp_min = daily.temperature_2m_min[i];
        assert!(
            temp_min > -50.0 && temp_min < 60.0,
            "Min temperature should be in reasonable range"
        );

        // Max should be >= min
        assert!(
            temp_max >= temp_min,
            "Max temperature should be >= min temperature"
        );

        // Verify precipitation
        let precip_sum = daily.precipitation_sum[i];
        assert!(
            (0.0..500.0).contains(&precip_sum),
            "Precipitation sum should be reasonable"
        );

        let precip_prob = daily.precipitation_probability_max[i];
        assert!(
            precip_prob <= 100,
            "Precipitation probability should be <= 100%"
        );
    }
}

/// Test Open-Meteo hourly forecasts are time-ordered
#[test]
fn test_open_meteo_hourly_time_ordering() {
    let json = fs::read_to_string("tests/fixtures/open_meteo_hourly_forecast.json")
        .expect("Failed to read Open-Meteo hourly forecast fixture file");

    let response: OpenMeteoHourlyResponse = serde_json::from_str(&json).unwrap();
    let hourly = &response.hourly;

    assert!(hourly.time.len() > 1, "Should have multiple forecast hours");

    // Verify time ordering
    for i in 1..hourly.time.len() {
        assert!(
            hourly.time[i] > hourly.time[i - 1],
            "Hourly forecasts should be in chronological order"
        );
    }
}

/// Test Open-Meteo daily forecasts are time-ordered
#[test]
fn test_open_meteo_daily_time_ordering() {
    let json = fs::read_to_string("tests/fixtures/open_meteo_daily_forecast.json")
        .expect("Failed to read Open-Meteo daily forecast fixture file");

    let response: OpenMeteoDailyResponse = serde_json::from_str(&json).unwrap();
    let daily = &response.daily;

    assert!(daily.time.len() > 1, "Should have multiple forecast days");

    // Verify time ordering
    for i in 1..daily.time.len() {
        assert!(
            daily.time[i] > daily.time[i - 1],
            "Daily forecasts should be in chronological order"
        );
    }
}

/// Test Open-Meteo coordinates are present
#[test]
fn test_open_meteo_coordinates() {
    let json = fs::read_to_string("tests/fixtures/open_meteo_hourly_forecast.json")
        .expect("Failed to read Open-Meteo hourly forecast fixture file");

    let response: OpenMeteoHourlyResponse = serde_json::from_str(&json).unwrap();

    // Verify latitude is reasonable
    assert!(
        response.latitude >= -90.0 && response.latitude <= 90.0,
        "Latitude should be between -90 and 90"
    );

    // Verify longitude is reasonable
    assert!(
        response.longitude >= -180.0 && response.longitude <= 180.0,
        "Longitude should be between -180 and 180"
    );
}
