//! Layer 1 Tests: BOM Provider JSON deserialization and conversion
//! 
//! These tests verify:
//! 1. BOM JSON fixtures can be loaded and parsed
//! 2. The data in the files has expected structure
//! 
//! Test fixtures are stored in tests/fixtures/ directory to avoid
//! dependency on runtime cache files from executing the binary.

use pi_inky_weather_epd::apis::bom::models::{
    HourlyForecastResponse,
    DailyForecastResponse,
};
use std::fs;

/// Test that BOM hourly forecast fixture deserializes
#[test]
fn test_load_bom_hourly_fixture() {
    let json = fs::read_to_string("tests/fixtures/bom_hourly_forecast.json")
        .expect("Failed to read BOM hourly forecast fixture file");
    
    let result: Result<HourlyForecastResponse, _> = serde_json::from_str(&json);
    assert!(result.is_ok(), "Failed to deserialize BOM hourly forecast: {:?}", result.err());
    
    let response = result.unwrap();
    assert!(response.data.len() > 0, "Expected at least one hourly forecast entry");
    
    // Validate first entry has reasonable data
    let first = &response.data[0];
    assert!(first.temp.value > -50.0 && first.temp.value < 60.0, "Temperature should be in reasonable range");
    assert!(first.wind.speed_kilometre < 500, "Wind speed should be reasonable");
}

/// Test that BOM daily forecast fixture deserializes
#[test]
fn test_load_bom_daily_fixture() {
    let json = fs::read_to_string("tests/fixtures/bom_daily_forecast.json")
        .expect("Failed to read BOM daily forecast fixture file");
    
    let result: Result<DailyForecastResponse, _> = serde_json::from_str(&json);
    assert!(result.is_ok(), "Failed to deserialize BOM daily forecast: {:?}", result.err());
    
    let response = result.unwrap();
    assert!(response.data.len() > 0, "Expected at least one daily forecast entry");
    
    // Validate first entry has reasonable data
    let first = &response.data[0];
    if let Some(temp_max) = &first.temp_max {
        assert!(temp_max.value > -50.0 && temp_max.value < 60.0, "Max temp should be in reasonable range");
    }
}

/// Test BOM hourly forecast has expected fields
#[test]
fn test_bom_hourly_fields() {
    let json = fs::read_to_string("tests/fixtures/bom_hourly_forecast.json")
        .expect("Failed to read BOM hourly forecast fixture file");
    
    let response: HourlyForecastResponse = serde_json::from_str(&json).unwrap();
    
    for forecast in &response.data {
        // Verify required fields exist
        assert!(forecast.time.timestamp() > 0, "Should have valid timestamp");
        assert!(forecast.temp.value.is_finite(), "Temperature should be finite");
        assert!(forecast.temp_feels_like.value.is_finite(), "Apparent temp should be finite");
        
        // Verify ranges are sensible
        if let Some(chance) = forecast.rain.chance {
            assert!(chance <= 100, "Rain chance should be <= 100%");
        }
        assert!(forecast.uv.0 < 20, "UV index should be < 20");
    }
}

/// Test BOM daily forecast has expected fields
#[test]
fn test_bom_daily_fields() {
    let json = fs::read_to_string("tests/fixtures/bom_daily_forecast.json")
        .expect("Failed to read BOM daily forecast fixture file");
    
    let response: DailyForecastResponse = serde_json::from_str(&json).unwrap();
    
    for entry in &response.data {
        // Verify temperature fields are sensible if present
        if let Some(temp_max) = &entry.temp_max {
            assert!(temp_max.value.is_finite(), "Max temp should be finite");
        }
        if let Some(temp_min) = &entry.temp_min {
            assert!(temp_min.value.is_finite(), "Min temp should be finite");
        }
        
        // Verify rain data if present
        if let Some(rain) = &entry.rain {
            if let Some(chance) = rain.chance {
                assert!(chance <= 100, "Rain chance should be <= 100%");
            }
        }
    }
}

/// Test BOM hourly forecasts are time-ordered
#[test]
fn test_bom_hourly_time_ordering() {
    let json = fs::read_to_string("tests/fixtures/bom_hourly_forecast.json")
        .expect("Failed to read BOM hourly forecast fixture file");
    
    let response: HourlyForecastResponse = serde_json::from_str(&json).unwrap();
    
    assert!(response.data.len() > 1, "Should have multiple forecast hours");
    
    // Verify time ordering
    for i in 1..response.data.len() {
        assert!(
            response.data[i].time > response.data[i-1].time,
            "Hourly forecasts should be in chronological order"
        );
    }
}
