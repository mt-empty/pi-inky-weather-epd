//! Wiremock helpers for Open-Meteo API tests
//!
//! This module provides helper functions to set up mock HTTP servers for testing
//! the Open-Meteo weather provider without making real API calls.

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
