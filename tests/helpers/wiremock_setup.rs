//! Wiremock helpers for weather API tests
//!
//! This module provides helper functions to set up mock HTTP servers for testing
//! weather providers without making real API calls.

use wiremock::{Mock, MockServer, ResponseTemplate};

/// Setup wiremock server for Open-Meteo API using separate hourly and daily fixture files
///
/// Open-Meteo API now uses two separate requests:
/// - Hourly endpoint: `timezone=UTC` for hourly forecast data
/// - Daily endpoint: `timezone=auto` for daily aggregations (max/min temps in local timezone)
///
/// # Arguments
/// * `hourly_fixture_path` - Path to hourly JSON (e.g., "tests/fixtures/open_meteo_hourly_forecast.json")
/// * `daily_fixture_path` - Path to daily JSON (e.g., "tests/fixtures/open_meteo_daily_forecast.json")
///
/// # Returns
/// Mock server instance - caller must keep this alive for the duration of the test
#[allow(dead_code)] // Used by Open-Meteo snapshot tests
pub async fn setup_open_meteo_mock(
    hourly_fixture_path: &str,
    daily_fixture_path: &str,
) -> MockServer {
    let mock_server = MockServer::start().await;

    // Load fixture data
    let hourly_fixture = std::fs::read_to_string(hourly_fixture_path).unwrap_or_else(|e| {
        panic!(
            "Failed to read hourly fixture from {}: {}",
            hourly_fixture_path, e
        )
    });
    let daily_fixture = std::fs::read_to_string(daily_fixture_path).unwrap_or_else(|e| {
        panic!(
            "Failed to read daily fixture from {}: {}",
            daily_fixture_path, e
        )
    });

    // Setup hourly endpoint - matches timezone=UTC
    Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/v1/forecast"))
        .and(wiremock::matchers::query_param("timezone", "UTC"))
        .respond_with(ResponseTemplate::new(200).set_body_string(hourly_fixture))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Setup daily endpoint - matches timezone=auto
    Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/v1/forecast"))
        .and(wiremock::matchers::query_param("timezone", "auto"))
        .respond_with(ResponseTemplate::new(200).set_body_string(daily_fixture))
        .expect(1)
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
