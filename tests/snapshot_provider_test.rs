//! Provider-specific snapshot tests
//!
//! These tests verify that the complete dashboard generation pipeline produces
//! consistent output for each weather provider (BOM and Open-Meteo).
//!
//! ## How These Tests Work
//!
//! 1. **Fixed Time**: Use FixedClock to ensure deterministic "current hour"
//! 2. **Fixture Data**: Load pre-defined weather data (no API calls)
//! 3. **Full Pipeline**: Run complete dashboard generation
//! 4. **Snapshot SVG**: Capture and compare the full SVG output
//!
//! ## Running These Tests
//!
//! **Default (Open-Meteo)**:
//! ```bash
//! TEST_MODE=config/test cargo test --test snapshot_provider_test
//! ```
//!
//! **BOM Provider** (requires override):
//! ```bash
//! TEST_MODE=config/test APP_API__PROVIDER=bom cargo test --test snapshot_provider_test snapshot_bom_dashboard -- --ignored
//! ```
//!
//! **Important**: Set `TEST_MODE=config/test` to load test configuration.
//! This loads `config/test.toml` which sets:
//! - `provider = "open_meteo"` (default test provider)
//! - `disable_weather_api_requests = true` (use fixtures, no real API calls)
//! - `weather_data_cache_path = "tests/fixtures/"` (load from test fixtures)
//!
//! ## Reviewing Snapshots
//!
//! On first run or after intentional changes:
//! ```bash
//! TEST_MODE=config/test cargo test --test snapshot_provider_test
//! cargo insta review  # Review and accept/reject changes
//! ```

use pi_inky_weather_epd::{clock::FixedClock, generate_weather_dashboard_with_clock, CONFIG};
use std::fs;

/// Test Open-Meteo provider dashboard generation with fixed time and fixtures
///
/// **Fixed Time**: Oct 10, 2025, 1:00 AM UTC = Oct 10, 2025, 12:00 PM Melbourne (AEDT)
///
/// **Fixture Files Used**:
/// - `tests/fixtures/open_meteo_forecast.json` - Combined hourly + daily data
///
/// **What This Tests**:
/// - Open-Meteo API response parsing (parallel array structure)
/// - Float temperature handling
/// - Global provider (lat/lon instead of geohash)
/// - GMT timezone conversion to Melbourne (AEDT)
/// - Dashboard rendering with Open-Meteo data structure
///
/// **Snapshot Captures**:
/// - Complete SVG structure (template + rendered data)
/// - Current weather display (temp, icon, feels like, wind, humidity, UV)
/// - Hourly forecast graph (temperature, rain)
/// - 7-day forecast cards (only 3 days in fixture)
/// - Sunrise/sunset times
///
/// **NOTE**: This is the default test provider (config/test.toml sets provider = "open_meteo").
/// No special environment variables needed.
#[test]
fn snapshot_open_meteo_dashboard() {
    // Verify test configuration is loaded
    assert!(
        CONFIG.debugging.disable_weather_api_requests,
        "Test requires TEST_MODE=config/test environment variable"
    );
    assert_eq!(
        CONFIG.misc.weather_data_cache_path.to_str().unwrap(),
        "tests/fixtures/",
        "Test config should use fixtures path"
    );
    assert_eq!(
        format!("{}", CONFIG.api.provider).to_lowercase(),
        "openmeteo",
        "Test config should use Open-Meteo provider"
    );

    // Fixed time: Oct 10, 2025, 1:00 AM UTC
    // In Melbourne AEDT (UTC+11): Oct 10, 2025, 12:00 PM (noon)
    // This should match mid-point of hourly forecast data
    let clock =
        FixedClock::from_rfc3339("2025-10-10T01:00:00Z").expect("Failed to create fixed clock");

    // Verify test configuration is loaded
    assert!(
        CONFIG.debugging.disable_weather_api_requests,
        "Test config should have disable_weather_api_requests = true"
    );

    // Temporarily override provider to Open-Meteo
    // Note: This requires the config to be reloadable or we need a different approach
    // For now, we'll test with whatever provider is set in test.toml

    // Generate the dashboard with Open-Meteo provider
    let result = generate_weather_dashboard_with_clock(&clock);
    assert!(
        result.is_ok(),
        "Failed to generate Open-Meteo dashboard: {:?}",
        result.err()
    );

    // Read the generated SVG
    let svg_content = fs::read_to_string(CONFIG.misc.generated_svg_name.clone())
        .expect("Failed to read generated SVG file");

    // Verify SVG is not empty
    assert!(!svg_content.is_empty(), "Generated SVG should not be empty");
    assert!(
        svg_content.contains("<svg"),
        "Generated file should be valid SVG"
    );

    // Snapshot the SVG structure
    // This captures the complete dashboard including:
    // - Weather data from Open-Meteo fixtures
    // - Rendered at fixed time (12:00 PM Melbourne)
    // - All graph paths, labels, icons
    insta::assert_snapshot!("open_meteo_dashboard", svg_content);
}

/// Test BOM provider dashboard generation with fixed time and fixtures
///
/// **Prerequisites**: Run with `TEST_MODE=config/test` environment variable
///
/// **Fixed Time**: Oct 9, 2025, 10:00 PM UTC = Oct 10, 2025, 9:00 AM Melbourne (AEDT)
///
/// **Fixture Files Used**:
/// - `tests/fixtures/bom_hourly_forecast.json` - 4 hours of data
/// - `tests/fixtures/bom_daily_forecast.json` - 3 days of data
///
/// **What This Tests**:
/// - BOM-specific API response parsing
/// - Integer temperature handling
/// - Knots to km/h wind conversion
/// - Australian timezone (AEDT) display
/// - Dashboard rendering with BOM data structure
///
/// **Snapshot Captures**:
/// - Complete SVG structure (template + rendered data)
/// - Current weather display (temp, icon, feels like, wind, humidity, UV)
/// - Hourly forecast graph (temperature, rain)
/// - 7-day forecast cards
/// - Sunrise/sunset times
///
/// **NOTE**: This test requires APP_API__PROVIDER environment variable override:
/// ```bash
/// TEST_MODE=config/test APP_API__PROVIDER=bom cargo test --test snapshot_provider_test snapshot_bom_dashboard -- --ignored
/// cargo insta review
/// ```
/// Note: Double underscore __ is used as separator for nested config keys (api.provider -> API__PROVIDER)
#[test]
#[ignore] // Run separately with APP_API__PROVIDER=bom (CONFIG initializes once per test binary)
fn snapshot_bom_dashboard() {
    // Verify test configuration is loaded (fail fast if TEST_MODE not set)
    assert!(
        CONFIG.debugging.disable_weather_api_requests,
        "Test requires TEST_MODE=config/test environment variable"
    );
    assert_eq!(
        CONFIG.misc.weather_data_cache_path.to_str().unwrap(),
        "tests/fixtures/",
        "Test config should use fixtures path"
    );
    assert_eq!(
        format!("{}", CONFIG.api.provider).to_lowercase(),
        "bom",
        "Test should use BOM provider (set APP_API__PROVIDER=bom)"
    );

    // Fixed time: Oct 9, 2025, 10:00 PM UTC
    // In Melbourne AEDT (UTC+11): Oct 10, 2025, 9:00 AM
    // This matches the first hourly forecast data point in the fixture
    let clock =
        FixedClock::from_rfc3339("2025-10-09T22:00:00Z").expect("Failed to create fixed clock");

    // Generate the dashboard with BOM provider
    let result = generate_weather_dashboard_with_clock(&clock);
    assert!(
        result.is_ok(),
        "Failed to generate BOM dashboard: {:?}",
        result.err()
    );

    // Read the generated SVG
    let svg_content = fs::read_to_string(CONFIG.misc.generated_svg_name.clone())
        .expect("Failed to read generated SVG file");

    // Verify SVG is not empty
    assert!(!svg_content.is_empty(), "Generated SVG should not be empty");
    assert!(
        svg_content.contains("<svg"),
        "Generated file should be valid SVG"
    );

    // Snapshot the SVG structure
    // This captures the complete dashboard including:
    // - Weather data from BOM fixtures
    // - Rendered at fixed time (9:00 AM Melbourne)
    // - All graph paths, labels, icons
    insta::assert_snapshot!("bom_dashboard", svg_content);
}
