//! Wiremock helpers for weather API tests
//!
//! This module provides helper functions to set up mock HTTP servers for testing
//! weather providers without making real API calls.

use wiremock::{Mock, MockServer, ResponseTemplate};

/// Setup wiremock server for Open-Meteo API using a fixture file
///
/// # Arguments
/// * `fixture_path` - Path to the JSON fixture file (e.g., "tests/fixtures/open_meteo_forecast.json")
///
/// # Returns
/// Mock server instance - caller must keep this alive for the duration of the test
#[allow(dead_code)] // Used by Open-Meteo snapshot tests
pub async fn setup_open_meteo_mock(fixture_path: &str) -> MockServer {
    let mock_server = MockServer::start().await;

    // Load fixture data
    let fixture_data = std::fs::read_to_string(fixture_path)
        .unwrap_or_else(|e| panic!("Failed to read fixture from {}: {}", fixture_path, e));

    // Setup mock endpoint - matches any GET request to /v1/forecast
    Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/v1/forecast"))
        .respond_with(ResponseTemplate::new(200).set_body_string(fixture_data))
        .expect(1) // Expect exactly one call per test
        .mount(&mock_server)
        .await;

    mock_server
}

/// Setup wiremock server for BOM API using fixture files
///
/// BOM API uses separate endpoints for daily and hourly forecasts:
/// - `/v1/locations/{geohash}/forecasts/daily`
/// - `/v1/locations/{geohash}/forecasts/hourly`
///
/// # Arguments
/// * `daily_fixture_path` - Path to daily forecast JSON (e.g., "tests/fixtures/bom_daily_forecast.json")
/// * `hourly_fixture_path` - Path to hourly forecast JSON (e.g., "tests/fixtures/bom_hourly_forecast.json")
///
/// # Returns
/// Mock server instance - caller must keep this alive for the duration of the test
#[allow(dead_code)] // Used by BOM snapshot tests
pub async fn setup_bom_mock(daily_fixture_path: &str, hourly_fixture_path: &str) -> MockServer {
    let mock_server = MockServer::start().await;

    // Load fixture data
    let daily_fixture = std::fs::read_to_string(daily_fixture_path).unwrap_or_else(|e| {
        panic!(
            "Failed to read daily fixture from {}: {}",
            daily_fixture_path, e
        )
    });
    let hourly_fixture = std::fs::read_to_string(hourly_fixture_path).unwrap_or_else(|e| {
        panic!(
            "Failed to read hourly fixture from {}: {}",
            hourly_fixture_path, e
        )
    });

    // Setup daily forecast endpoint - matches any geohash
    Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path_regex(
            r"/v1/locations/[^/]+/forecasts/daily",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_string(daily_fixture))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Setup hourly forecast endpoint - matches any geohash
    Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path_regex(
            r"/v1/locations/[^/]+/forecasts/hourly",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_string(hourly_fixture))
        .expect(1)
        .mount(&mock_server)
        .await;

    mock_server
}
