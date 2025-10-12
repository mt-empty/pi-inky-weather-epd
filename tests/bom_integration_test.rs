//! Layer 2 Integration Tests: BOM Provider Behavior and Domain Conversion
//! 
//! These tests verify:
//! 1. Conversion from BOM API models to domain models
//! 2. Edge cases and optional field handling
//! 3. Temperature conversion logic
//! 4. Data transformation correctness

use pi_inky_weather_epd::apis::bom::models::{
    HourlyForecastResponse,
    DailyForecastResponse,
};
use pi_inky_weather_epd::domain::models::{HourlyForecast, DailyForecast};
use std::fs;

/// Test conversion from BOM hourly response to domain models
#[test]
fn test_bom_hourly_to_domain_conversion() {
    let json = fs::read_to_string("tests/fixtures/bom_hourly_forecast.json")
        .expect("Failed to read BOM hourly forecast fixture");
    
    let response: HourlyForecastResponse = serde_json::from_str(&json).unwrap();
    let bom_data = response.data;
    
    // Convert to domain models
    let domain_forecasts: Vec<HourlyForecast> = bom_data
        .into_iter()
        .map(|bom| bom.into())
        .collect();
    
    // Verify conversion happened
    assert!(domain_forecasts.len() > 0, "Should have converted forecasts");
    
    // Spot check first forecast
    let first = &domain_forecasts[0];
    assert!(first.temperature.value > -50.0 && first.temperature.value < 60.0);
    assert!(first.wind.speed_kmh < 500);
    assert!(first.uv_index < 20);
}

/// Test conversion from BOM daily response to domain models
#[test]
fn test_bom_daily_to_domain_conversion() {
    let json = fs::read_to_string("tests/fixtures/bom_daily_forecast.json")
        .expect("Failed to read BOM daily forecast fixture");
    
    let response: DailyForecastResponse = serde_json::from_str(&json).unwrap();
    let bom_data = response.data;
    
    // Convert to domain models
    let domain_forecasts: Vec<DailyForecast> = bom_data
        .into_iter()
        .map(|bom| bom.into())
        .collect();
    
    // Verify conversion happened
    assert!(domain_forecasts.len() > 0, "Should have converted forecasts");
    
    // Spot check first forecast
    let first = &domain_forecasts[0];
    if let Some(temp_max) = first.temp_max {
        assert!(temp_max.value > -50.0 && temp_max.value < 60.0);
    }
    if let Some(temp_min) = first.temp_min {
        assert!(temp_min.value > -50.0 && temp_min.value < 60.0);
    }
}

/// Test BOM handles optional precipitation amounts correctly
#[test]
fn test_bom_precipitation_edge_cases() {
    // Create fixture with edge cases
    let json = r#"{
        "data": [{
            "rain": {
                "amount": {"min": null, "max": null, "units": "mm"},
                "chance": 0
            },
            "temp": 20,
            "temp_feels_like": 18,
            "wind": {
                "speed_knot": 10,
                "speed_kilometre": 18,
                "direction": "N",
                "gust_speed_knot": 15,
                "gust_speed_kilometre": 28
            },
            "relative_humidity": 50,
            "uv": 5,
            "time": "2025-10-10T12:00:00Z",
            "is_night": false
        }]
    }"#;
    
    let response: HourlyForecastResponse = serde_json::from_str(&json).unwrap();
    let domain: Vec<HourlyForecast> = response.data
        .into_iter()
        .map(|bom| bom.into())
        .collect();
    
    let forecast = &domain[0];
    
    // Verify null amounts are converted to None
    assert_eq!(forecast.precipitation.amount_min, None);
    assert_eq!(forecast.precipitation.amount_max, None);
    assert_eq!(forecast.precipitation.chance, Some(0));
}

/// Test BOM extreme weather values are preserved through conversion
#[test]
fn test_bom_extreme_weather_conversion() {
    let json = r#"{
        "data": [{
            "rain": {
                "amount": {"min": 50, "max": 100, "units": "mm"},
                "chance": 100
            },
            "temp": 45,
            "temp_feels_like": 50,
            "wind": {
                "speed_knot": 60,
                "speed_kilometre": 111,
                "direction": "S",
                "gust_speed_knot": 80,
                "gust_speed_kilometre": 148
            },
            "relative_humidity": 95,
            "uv": 14,
            "time": "2025-10-10T12:00:00Z",
            "is_night": false
        }]
    }"#;
    
    let response: HourlyForecastResponse = serde_json::from_str(&json).unwrap();
    let domain: Vec<HourlyForecast> = response.data
        .into_iter()
        .map(|bom| bom.into())
        .collect();
    
    let forecast = &domain[0];
    
    // Verify extreme values are preserved
    assert_eq!(forecast.temperature.value, 45.0);
    assert_eq!(forecast.apparent_temperature.value, 50.0);
    assert_eq!(forecast.precipitation.chance, Some(100));
    assert_eq!(forecast.precipitation.amount_min, Some(50));
    assert_eq!(forecast.precipitation.amount_max, Some(100));
    assert_eq!(forecast.wind.speed_kmh, 111);
    assert_eq!(forecast.wind.gust_speed_kmh, 148);
    assert_eq!(forecast.uv_index, 14);
}

/// Test BOM daily forecast with missing optional temperature fields
#[test]
fn test_bom_daily_missing_temps() {
    let json = r#"{
        "data": [{
            "temp_max": 25,
            "temp_min": null,
            "rain": {
                "amount": {"min": null, "max": null, "units": "mm"},
                "chance": 10
            },
            "astronomical": {
                "sunrise_time": "2025-10-10T20:00:00Z",
                "sunset_time": "2025-10-11T09:00:00Z"
            },
            "date": "2025-10-10T14:00:00Z"
        }]
    }"#;
    
    let response: DailyForecastResponse = serde_json::from_str(&json).unwrap();
    let domain: Vec<DailyForecast> = response.data
        .into_iter()
        .map(|bom| bom.into())
        .collect();
    
    let forecast = &domain[0];
    
    // Verify temp_max is present, temp_min is None
    assert!(forecast.temp_max.is_some());
    assert_eq!(forecast.temp_max.unwrap().value, 25.0);
    assert!(forecast.temp_min.is_none());
}

/// Test BOM converts all hourly forecasts preserving order
#[test]
fn test_bom_hourly_conversion_preserves_order() {
    let json = fs::read_to_string("tests/fixtures/bom_hourly_forecast.json")
        .expect("Failed to read BOM hourly forecast fixture");
    
    let response: HourlyForecastResponse = serde_json::from_str(&json).unwrap();
    let expected_count = response.data.len();
    
    let domain_forecasts: Vec<HourlyForecast> = response.data
        .into_iter()
        .map(|bom| bom.into())
        .collect();
    
    // Verify same number of forecasts
    assert_eq!(domain_forecasts.len(), expected_count);
    
    // Verify chronological order is preserved
    for i in 1..domain_forecasts.len() {
        assert!(
            domain_forecasts[i].time > domain_forecasts[i-1].time,
            "Order should be preserved after conversion"
        );
    }
}
