//! Layer 2 Integration Tests: Open-Meteo Provider Behavior and Domain Conversion
//!
//! These tests verify:
//! 1. Conversion from Open-Meteo API models to domain models
//! 2. Array-to-struct transformation correctness
//! 3. Hourly and daily data extraction from combined response
//! 4. Edge cases and data consistency

use pi_inky_weather_epd::apis::open_meteo::models::{
    OpenMeteoDailyResponse, OpenMeteoHourlyResponse,
};
use pi_inky_weather_epd::domain::models::{DailyForecast, HourlyForecast};
use std::fs;

/// Test conversion from Open-Meteo response to hourly domain models
#[test]
fn test_open_meteo_hourly_conversion() {
    let json = fs::read_to_string("tests/fixtures/open_meteo_hourly_forecast.json")
        .expect("Failed to read Open-Meteo hourly forecast fixture");

    let response: OpenMeteoHourlyResponse = serde_json::from_str(&json).unwrap();
    let expected_count = response.hourly.time.len();

    // Convert to domain models
    let domain_forecasts: Vec<HourlyForecast> = response.into();

    // Verify conversion happened
    assert_eq!(
        domain_forecasts.len(),
        expected_count,
        "Should convert all hourly entries"
    );

    // Spot check first forecast
    let first = &domain_forecasts[0];
    assert!(first.temperature.value > -50.0 && first.temperature.value < 60.0);
    assert!(first.wind.speed_kmh < 500);
    assert!(first.uv_index < 20);
}

/// Test conversion from Open-Meteo response to daily domain models
#[test]
fn test_open_meteo_daily_conversion() {
    let json = fs::read_to_string("tests/fixtures/open_meteo_daily_forecast.json")
        .expect("Failed to read Open-Meteo daily forecast fixture");

    let response: OpenMeteoDailyResponse = serde_json::from_str(&json).unwrap();
    let expected_count = response.daily.time.len();

    // Convert to domain models
    let domain_forecasts: Vec<DailyForecast> = response.into();

    // Verify conversion happened
    assert_eq!(
        domain_forecasts.len(),
        expected_count,
        "Should convert all daily entries"
    );

    // Spot check first forecast
    let first = &domain_forecasts[0];
    assert!(first.temp_max.is_some());
    assert!(first.temp_min.is_some());

    if let (Some(max), Some(min)) = (first.temp_max, first.temp_min) {
        assert!(max.value >= min.value, "Max temp should be >= min temp");
    }
}

/// Test Open-Meteo array transformation creates consistent structs
#[test]
fn test_open_meteo_array_consistency() {
    let json = r#"{
        "latitude": -37.75,
        "longitude": 144.875,
        "timezone": "GMT",
        "timezone_abbreviation": "GMT",
        "current_units": {"time": "iso8601", "interval": "seconds", "is_day": ""},
        "current": {"time": "2025-10-10T12:00", "interval": 900, "is_day": 1},
        "hourly_units": {
            "time": "iso8601",
            "temperature_2m": "°C",
            "apparent_temperature": "°C",
            "precipitation_probability": "%",
            "precipitation": "mm","snowfall":"cm",
            "uv_index": "",
            "wind_speed_10m": "km/h",
            "wind_gusts_10m": "km/h",
            "relative_humidity_2m": "%"
        },
        "hourly": {
            "time": ["2025-10-10T00:00", "2025-10-10T01:00"],
            "temperature_2m": [18.5, 19.2],
            "apparent_temperature": [15.1, 16.0],
            "precipitation_probability": [10, 20],
            "precipitation": [0.0, 0.5],"snowfall":[0.0,0.0],
            "uv_index": [0.0, 0.0],
            "wind_speed_10m": [15.0, 18.0],
            "wind_gusts_10m": [25.0, 30.0],
            "relative_humidity_2m": [65, 70]
            ,"cloud_cover": [30, 45]
        },
        "daily_units": {
            "time": "iso8601",
            "sunrise": "iso8601",
            "sunset": "iso8601",
            "temperature_2m_max": "°C",
            "temperature_2m_min": "°C",
            "precipitation_sum": "mm",
            "precipitation_probability_max": "%","snowfall_sum":"cm"
        },
        "daily": {
            "time": ["2025-10-10"],
            "sunrise": ["2025-10-10T06:00"],
            "sunset": ["2025-10-10T18:00"],
            "temperature_2m_max": [25.0],
            "temperature_2m_min": [12.0],
            "precipitation_sum": [2.5],
            "precipitation_probability_max": [60],"snowfall_sum":[0.0]
            ,"cloud_cover_mean": [55]
        }
    }"#;

    let response: OpenMeteoHourlyResponse = serde_json::from_str(json).unwrap();
    let domain: Vec<HourlyForecast> = response.into();

    // Verify each hourly forecast has correct values from arrays
    assert_eq!(domain.len(), 2);

    assert_eq!(domain[0].temperature.value, 18.5);
    assert_eq!(domain[0].apparent_temperature.value, 15.1);
    assert_eq!(domain[0].precipitation.chance, Some(10));
    assert_eq!(domain[0].wind.speed_kmh, 15);

    assert_eq!(domain[1].temperature.value, 19.2);
    assert_eq!(domain[1].apparent_temperature.value, 16.0);
    assert_eq!(domain[1].precipitation.chance, Some(20));
    assert_eq!(domain[1].wind.speed_kmh, 18);
}

/// Test Open-Meteo extreme values are preserved
#[test]
fn test_open_meteo_extreme_values() {
    let json = r#"{
        "latitude": -37.75,
        "longitude": 144.875,
        "timezone": "GMT",
        "timezone_abbreviation": "GMT",
        "current_units": {"time": "iso8601", "interval": "seconds", "is_day": ""},
        "current": {"time": "2025-10-10T12:00", "interval": 900, "is_day": 1},
        "hourly_units": {
            "time": "iso8601",
            "temperature_2m": "°C",
            "apparent_temperature": "°C",
            "precipitation_probability": "%",
            "precipitation": "mm","snowfall":"cm",
            "uv_index": "",
            "wind_speed_10m": "km/h",
            "wind_gusts_10m": "km/h",
            "relative_humidity_2m": "%"
        },
        "hourly": {
            "time": ["2025-10-10T12:00"],
            "temperature_2m": [48.5],
            "apparent_temperature": [55.0],
            "precipitation_probability": [100],
            "precipitation": [150.0],"snowfall":[0.0],
            "uv_index": [15],
            "wind_speed_10m": [120.0],
            "wind_gusts_10m": [180.0],
            "relative_humidity_2m": [99]
            ,"cloud_cover": [95]
        },
        "daily_units": {
            "time": "iso8601",
            "sunrise": "iso8601",
            "sunset": "iso8601",
            "temperature_2m_max": "°C",
            "temperature_2m_min": "°C",
            "precipitation_sum": "mm",
            "precipitation_probability_max": "%","snowfall_sum":"cm"
        },
        "daily": {
            "time": ["2025-10-10"],
            "sunrise": ["2025-10-10T06:00"],
            "sunset": ["2025-10-10T18:00"],
            "temperature_2m_max": [50.0],
            "temperature_2m_min": [-10.0],
            "precipitation_sum": [200.0],
            "precipitation_probability_max": [100],"snowfall_sum":[0.0]
            ,"cloud_cover_mean": [98]
        }
    }"#;

    let response: OpenMeteoHourlyResponse = serde_json::from_str(json).unwrap();
    let domain: Vec<HourlyForecast> = response.into();

    let forecast = &domain[0];

    // Verify extreme values are preserved
    assert_eq!(forecast.temperature.value, 48.5);
    assert_eq!(forecast.apparent_temperature.value, 55.0);
    assert_eq!(forecast.precipitation.chance, Some(100));
    assert_eq!(forecast.wind.speed_kmh, 120);
    assert_eq!(forecast.wind.gust_speed_kmh, 180);
    assert_eq!(forecast.uv_index, 15);
}

/// Test Open-Meteo daily conversion preserves min/max relationship
#[test]
fn test_open_meteo_daily_temp_relationship() {
    let json = fs::read_to_string("tests/fixtures/open_meteo_daily_forecast.json")
        .expect("Failed to read Open-Meteo daily forecast fixture");

    let response: OpenMeteoDailyResponse = serde_json::from_str(&json).unwrap();
    let domain_forecasts: Vec<DailyForecast> = response.into();

    // Verify every daily forecast has max >= min
    for forecast in &domain_forecasts {
        if let (Some(max), Some(min)) = (forecast.temp_max, forecast.temp_min) {
            assert!(
                max.value >= min.value,
                "Daily max temp ({}) should be >= min temp ({})",
                max.value,
                min.value
            );
        }
    }
}

/// Test Open-Meteo handles zero precipitation correctly
#[test]
fn test_open_meteo_zero_precipitation() {
    let json = r#"{
        "latitude": -37.75,
        "longitude": 144.875,
        "timezone": "GMT",
        "timezone_abbreviation": "GMT",
        "current_units": {"time": "iso8601", "interval": "seconds", "is_day": ""},
        "current": {"time": "2025-10-10T12:00", "interval": 900, "is_day": 1},
        "hourly_units": {
            "time": "iso8601",
            "temperature_2m": "°C",
            "apparent_temperature": "°C",
            "precipitation_probability": "%",
            "precipitation": "mm","snowfall":"cm",
            "uv_index": "",
            "wind_speed_10m": "km/h",
            "wind_gusts_10m": "km/h",
            "relative_humidity_2m": "%"
        },
        "hourly": {
            "time": ["2025-10-10T12:00"],
            "temperature_2m": [20.0],
            "apparent_temperature": [18.0],
            "precipitation_probability": [0],
            "precipitation": [0.0],"snowfall":[0.0],
            "uv_index": [5.0],
            "wind_speed_10m": [10.0],
            "wind_gusts_10m": [15.0],
            "relative_humidity_2m": [50]
            ,"cloud_cover": [10]
        },
        "daily_units": {
            "time": "iso8601",
            "sunrise": "iso8601",
            "sunset": "iso8601",
            "temperature_2m_max": "°C",
            "temperature_2m_min": "°C",
            "precipitation_sum": "mm",
            "precipitation_probability_max": "%","snowfall_sum":"cm"
        },
        "daily": {
            "time": ["2025-10-10"],
            "sunrise": ["2025-10-10T06:00"],
            "sunset": ["2025-10-10T18:00"],
            "temperature_2m_max": [25.0],
            "temperature_2m_min": [15.0],
            "precipitation_sum": [0.0],
            "precipitation_probability_max": [0],"snowfall_sum":[0.0]
            ,"cloud_cover_mean": [5]
        }
    }"#;

    let response: OpenMeteoHourlyResponse = serde_json::from_str(json).unwrap();
    let domain: Vec<HourlyForecast> = response.into();

    let forecast = &domain[0];

    // Verify zero precipitation is handled
    assert_eq!(forecast.precipitation.chance, Some(0));
    // Note: Open-Meteo doesn't provide min/max per hour, just total
}

/// Test Open-Meteo conversion preserves chronological order
#[test]
fn test_open_meteo_conversion_preserves_order() {
    let json = fs::read_to_string("tests/fixtures/open_meteo_hourly_forecast.json")
        .expect("Failed to read Open-Meteo hourly forecast fixture");

    let response: OpenMeteoHourlyResponse = serde_json::from_str(&json).unwrap();
    let domain_forecasts: Vec<HourlyForecast> = response.into();

    // Verify chronological order is preserved
    for i in 1..domain_forecasts.len() {
        assert!(
            domain_forecasts[i].time > domain_forecasts[i - 1].time,
            "Order should be preserved after conversion"
        );
    }
}

/// Test hourly forecast conversion preserves snowfall data
#[test]
fn test_hourly_forecast_conversion_preserves_snowfall() {
    // Test that snowfall data flows from API response -> domain model
    let json = fs::read_to_string("tests/fixtures/open_meteo_hourly_forecast.json")
        .expect("Failed to read Open-Meteo hourly forecast fixture");

    let response: OpenMeteoHourlyResponse = serde_json::from_str(&json).unwrap();
    let hourly_forecasts: Vec<HourlyForecast> = response.into();

    // Find an entry with snowfall
    let with_snow = hourly_forecasts.iter().find(|f| f.precipitation.has_snow());

    assert!(
        with_snow.is_some(),
        "Expected at least one forecast with snowfall in test fixture"
    );

    // Verify the snowfall data is properly converted
    if let Some(forecast) = with_snow {
        let snowfall = forecast.precipitation.snowfall_amount.unwrap();
        assert!(
            snowfall > 0,
            "Snowfall amount should be greater than 0 for entries with snow"
        );
    }
}

/// Test daily forecast conversion preserves snowfall sum
#[test]
fn test_daily_forecast_conversion_preserves_snowfall() {
    // Test that snowfall_sum data flows from API response -> domain model
    let json = fs::read_to_string("tests/fixtures/open_meteo_daily_forecast.json")
        .expect("Failed to read Open-Meteo daily forecast fixture");

    let response: OpenMeteoDailyResponse = serde_json::from_str(&json).unwrap();
    let daily_forecasts: Vec<DailyForecast> = response.into();

    // Find an entry with snowfall
    let with_snow = daily_forecasts.iter().find(|f| {
        f.precipitation
            .as_ref()
            .map(|p| p.has_snow())
            .unwrap_or(false)
    });

    assert!(
        with_snow.is_some(),
        "Expected at least one daily forecast with snowfall in test fixture"
    );
}

/// Test snowfall data with zero values is handled correctly
#[test]
fn test_zero_snowfall_handled_correctly() {
    let json = r#"{
        "latitude": 45.38,
        "longitude": -81.109,
        "timezone": "America/Toronto",
        "timezone_abbreviation": "EST",
        "current_units": {"time": "iso8601", "interval": "seconds", "is_day": ""},
        "current": {"time": "2026-01-18T12:00", "interval": 900, "is_day": 1},
        "hourly_units": {
            "time": "iso8601",
            "temperature_2m": "°C",
            "apparent_temperature": "°C",
            "precipitation_probability": "%",
            "precipitation": "mm",
            "snowfall": "cm",
            "wind_speed_10m": "km/h",
            "wind_gusts_10m": "km/h",
            "uv_index": "",
            "relative_humidity_2m": "%",
            "cloud_cover": "%",
            "is_day": ""
        },
        "hourly": {
            "time": ["2026-01-18T12:00"],
            "temperature_2m": [5.0],
            "apparent_temperature": [3.0],
            "precipitation_probability": [20],
            "precipitation": [1.0],
            "snowfall": [0.0],
            "wind_speed_10m": [10],
            "wind_gusts_10m": [15],
            "uv_index": [2],
            "relative_humidity_2m": [70],
            "cloud_cover": [40],
            "is_day": [1]
        }
    }"#;

    let response: OpenMeteoHourlyResponse = serde_json::from_str(json).unwrap();
    let domain: Vec<HourlyForecast> = response.into();

    assert_eq!(domain.len(), 1);
    assert_eq!(domain[0].precipitation.snowfall_amount, Some(0));
    assert!(!domain[0].precipitation.has_snow());
    assert!(!domain[0].precipitation.is_primarily_snow());
}
