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
//! 5. **Auto-Skip**: Tests automatically skip if wrong provider is configured
//!
//! ## Running These Tests
//!
//! **Test Open-Meteo provider** (default in test.toml):
//! ```bash
//! RUN_MODE=test cargo test --test snapshot_provider_test
//! # Explicitly:
//! RUN_MODE=test APP_API__PROVIDER=open_meteo cargo test --test snapshot_provider_test
//! ```
//!
//! **Test BOM provider**:
//! ```bash
//! RUN_MODE=test APP_API__PROVIDER=bom cargo test --test snapshot_provider_test
//! ```
//!
//! **Test both providers** (for CI or comprehensive testing):
//! ```bash
//! RUN_MODE=test APP_API__PROVIDER=open_meteo cargo test --test snapshot_provider_test
//! RUN_MODE=test APP_API__PROVIDER=bom cargo test --test snapshot_provider_test
//! ```
//!
//! **Important**: Set `RUN_MODE=test` to load test configuration.
//! This loads `config/test.toml` which sets:
//! - `disable_weather_api_requests = true` (use fixtures, no real API calls)
//! - `weather_data_cache_path = "tests/fixtures/"` (load from test fixtures)
//!
//! ## Reviewing Snapshots
//!
//! On first run or after intentional changes:
//! ```bash
//! RUN_MODE=test APP_API__PROVIDER=open_meteo cargo test --test snapshot_provider_test
//! RUN_MODE=test APP_API__PROVIDER=bom cargo test --test snapshot_provider_test
//! cargo insta review  # Review and accept/reject changes
//! ```

use pi_inky_weather_epd::{clock::FixedClock, generate_weather_dashboard_with_clock, CONFIG};
use std::fs;

/// Test Open-Meteo provider dashboard generation with fixed time and fixtures
///
/// **Fixed Time**: Oct 25, 2025, 1:00 AM UTC = Oct 25, 2025, 12:00 PM Melbourne (AEDT)
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
/// **Running This Test**:
/// ```bash
/// RUN_MODE=test cargo test --test snapshot_provider_test
/// # Or explicitly with Open-Meteo:
/// RUN_MODE=test APP_API__PROVIDER=open_meteo cargo test --test snapshot_provider_test
/// ```
#[test]
fn snapshot_open_meteo_dashboard() {
    // Verify test configuration is loaded
    assert!(
        CONFIG.debugging.disable_weather_api_requests,
        "Test requires RUN_MODE=test environment variable"
    );
    assert_eq!(
        CONFIG.misc.weather_data_cache_path.to_str().unwrap(),
        "tests/fixtures/",
        "Test config should use fixtures path"
    );

    // Skip test if BOM provider is configured (test is provider-specific)
    if format!("{}", CONFIG.api.provider).to_lowercase() != "openmeteo" {
        eprintln!(
            "Skipping Open-Meteo test - provider is set to '{}'",
            CONFIG.api.provider
        );
        return;
    }

    // Fixed time: Oct 25, 2025, 1:00 AM UTC
    // In Melbourne AEDT (UTC+11): Oct 25, 2025, 12:00 PM (noon)
    // This should match mid-point of hourly forecast data
    let clock =
        FixedClock::from_rfc3339("2025-10-25T01:00:00Z").expect("Failed to create fixed clock");

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

/// Test Open-Meteo at midnight boundary (date transition edge case)
///
/// **Fixed Time**: Oct 26, 2025, 00:00:00 UTC = Oct 26, 2025, 11:00 AM Melbourne (AEDT)
///
/// **Edge Case**: Tests dashboard generation exactly at midnight UTC, which is mid-day in Melbourne.
/// Verifies that:
/// - Date transitions don't cause off-by-one errors
/// - Daily forecast shows correct days (Sun-Sat)
/// - Hourly graph starts from the correct hour
/// - Sunrise/sunset times are properly associated with the right date
#[test]
fn snapshot_open_meteo_midnight_boundary() {
    assert!(
        CONFIG.debugging.disable_weather_api_requests,
        "Test requires RUN_MODE=test environment variable"
    );

    if format!("{}", CONFIG.api.provider).to_lowercase() != "openmeteo" {
        eprintln!("Skipping Open-Meteo midnight test - provider is set to '{}'", CONFIG.api.provider);
        return;
    }

    // Midnight UTC on Oct 26 = 11:00 AM Melbourne
    let clock = FixedClock::from_rfc3339("2025-10-26T00:00:00Z")
        .expect("Failed to create fixed clock");

    let result = generate_weather_dashboard_with_clock(&clock);
    assert!(result.is_ok(), "Dashboard generation failed: {:?}", result.err());

    let svg_content = fs::read_to_string(CONFIG.misc.generated_svg_name.clone())
        .expect("Failed to read generated SVG file");

    assert!(!svg_content.is_empty() && svg_content.contains("<svg"));
    insta::assert_snapshot!("open_meteo_midnight_boundary", svg_content);
}

/// Test Open-Meteo at end of day (late evening edge case)
///
/// **Fixed Time**: Oct 25, 2025, 13:00:00 UTC = Oct 26, 2025, 12:00 AM Melbourne (AEDT)
///
/// **Edge Case**: Tests dashboard at midnight local time (just rolled over to new day).
/// Verifies that:
/// - "Today" refers to the new local date (Oct 26)
/// - Yesterday's data is not incorrectly shown as today
/// - Daily forecasts align with local calendar days
#[test]
fn snapshot_open_meteo_end_of_day() {
    assert!(
        CONFIG.debugging.disable_weather_api_requests,
        "Test requires RUN_MODE=test environment variable"
    );

    if format!("{}", CONFIG.api.provider).to_lowercase() != "openmeteo" {
        eprintln!("Skipping Open-Meteo end-of-day test - provider is set to '{}'", CONFIG.api.provider);
        return;
    }

    // 13:00 UTC = Midnight Melbourne (just rolled over to Oct 26)
    let clock = FixedClock::from_rfc3339("2025-10-25T13:00:00Z")
        .expect("Failed to create fixed clock");

    let result = generate_weather_dashboard_with_clock(&clock);
    assert!(result.is_ok(), "Dashboard generation failed: {:?}", result.err());

    let svg_content = fs::read_to_string(CONFIG.misc.generated_svg_name.clone())
        .expect("Failed to read generated SVG file");

    assert!(!svg_content.is_empty() && svg_content.contains("<svg"));
    insta::assert_snapshot!("open_meteo_end_of_day", svg_content);
}

/// Test Open-Meteo during DST transition period
///
/// **Fixed Time**: Oct 25, 2025, 16:00:00 UTC = Oct 26, 2025, 3:00 AM Melbourne (AEDT)
///
/// **Edge Case**: Tests during early morning hours (3 AM local).
/// Verifies that:
/// - Timezone offset (UTC+11 for AEDT) is correctly applied
/// - Hour labels in graph start from 3 AM
/// - No hour is skipped or duplicated in the 24-hour window
#[test]
fn snapshot_open_meteo_early_morning() {
    assert!(
        CONFIG.debugging.disable_weather_api_requests,
        "Test requires RUN_MODE=test environment variable"
    );

    if format!("{}", CONFIG.api.provider).to_lowercase() != "openmeteo" {
        eprintln!("Skipping Open-Meteo early morning test - provider is set to '{}'", CONFIG.api.provider);
        return;
    }

    // 16:00 UTC = 3:00 AM Melbourne
    let clock = FixedClock::from_rfc3339("2025-10-25T16:00:00Z")
        .expect("Failed to create fixed clock");

    let result = generate_weather_dashboard_with_clock(&clock);
    assert!(result.is_ok(), "Dashboard generation failed: {:?}", result.err());

    let svg_content = fs::read_to_string(CONFIG.misc.generated_svg_name.clone())
        .expect("Failed to read generated SVG file");

    assert!(!svg_content.is_empty() && svg_content.contains("<svg"));
    insta::assert_snapshot!("open_meteo_early_morning", svg_content);
}


/// Test BOM provider dashboard generation with fixed time and fixtures
///
/// **Fixed Time**: Oct 25, 2025, 10:00 AM UTC = Oct 25, 2025, 9:00 PM Melbourne (AEDT)
///
/// **Fixture Files Used**:
/// - `tests/fixtures/bom_hourly_forecast.json` - Starts at 11:00 UTC
/// - `tests/fixtures/bom_daily_forecast.json` - Starts Oct 24
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
/// **Running This Test**:
/// ```bash
/// RUN_MODE=test APP_API__PROVIDER=bom cargo test --test snapshot_provider_test
/// cargo insta review
/// ```
/// Note: Double underscore __ is used as separator for nested config keys (api.provider -> API__PROVIDER)
#[test]
fn snapshot_bom_dashboard() {
    // Verify test configuration is loaded (fail fast if TEST_MODE not set)
    assert!(
        CONFIG.debugging.disable_weather_api_requests,
        "Test requires RUN_MODE=test environment variable"
    );
    assert_eq!(
        CONFIG.misc.weather_data_cache_path.to_str().unwrap(),
        "tests/fixtures/",
        "Test config should use fixtures path"
    );

    // Skip test if Open-Meteo provider is configured (test is provider-specific)
    if format!("{}", CONFIG.api.provider).to_lowercase() != "bom" {
        eprintln!(
            "Skipping BOM test - provider is set to '{}'",
            CONFIG.api.provider
        );
        return;
    }

    // Fixed time: Oct 25, 2025, 10:00 AM UTC
    // In Melbourne AEDT (UTC+11): Oct 25, 2025, 9:00 PM
    // This is just before the first hourly forecast (11:00 UTC) in the fixture
    let clock =
        FixedClock::from_rfc3339("2025-10-25T10:00:00Z").expect("Failed to create fixed clock");

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

/// Test BOM at midnight boundary (date transition edge case)
///
/// **Fixed Time**: Oct 26, 2025, 00:00:00 UTC = Oct 26, 2025, 11:00 AM Melbourne (AEDT)
///
/// **Edge Case**: Midnight UTC boundary for BOM provider.
/// Verifies BOM-specific parsing handles date transitions correctly.
#[test]
fn snapshot_bom_midnight_boundary() {
    assert!(
        CONFIG.debugging.disable_weather_api_requests,
        "Test requires RUN_MODE=test environment variable"
    );

    if format!("{}", CONFIG.api.provider).to_lowercase() != "bom" {
        eprintln!("Skipping BOM midnight test - provider is set to '{}'", CONFIG.api.provider);
        return;
    }

    // Midnight UTC on Oct 26 = 11:00 AM Melbourne
    let clock = FixedClock::from_rfc3339("2025-10-26T00:00:00Z")
        .expect("Failed to create fixed clock");

    let result = generate_weather_dashboard_with_clock(&clock);
    assert!(result.is_ok(), "Dashboard generation failed: {:?}", result.err());

    let svg_content = fs::read_to_string(CONFIG.misc.generated_svg_name.clone())
        .expect("Failed to read generated SVG file");

    assert!(!svg_content.is_empty() && svg_content.contains("<svg"));
    insta::assert_snapshot!("bom_midnight_boundary", svg_content);
}

/// Test BOM at local midnight (just after day rollover)
///
/// **Fixed Time**: Oct 25, 2025, 13:00:00 UTC = Oct 26, 2025, 12:00 AM Melbourne (AEDT)
///
/// **Edge Case**: Tests BOM provider at local midnight.
/// Verifies daily forecast alignment with Australian calendar days.
#[test]
fn snapshot_bom_local_midnight() {
    assert!(
        CONFIG.debugging.disable_weather_api_requests,
        "Test requires RUN_MODE=test environment variable"
    );

    if format!("{}", CONFIG.api.provider).to_lowercase() != "bom" {
        eprintln!("Skipping BOM local midnight test - provider is set to '{}'", CONFIG.api.provider);
        return;
    }

    // 13:00 UTC = Midnight Melbourne (Oct 26)
    let clock = FixedClock::from_rfc3339("2025-10-25T13:00:00Z")
        .expect("Failed to create fixed clock");

    let result = generate_weather_dashboard_with_clock(&clock);
    assert!(result.is_ok(), "Dashboard generation failed: {:?}", result.err());

    let svg_content = fs::read_to_string(CONFIG.misc.generated_svg_name.clone())
        .expect("Failed to read generated SVG file");

    assert!(!svg_content.is_empty() && svg_content.contains("<svg"));
    insta::assert_snapshot!("bom_local_midnight", svg_content);
}

/// Test BOM during early morning hours
///
/// **Fixed Time**: Oct 25, 2025, 19:00:00 UTC = Oct 26, 2025, 6:00 AM Melbourne (AEDT)
///
/// **Edge Case**: Tests BOM provider at 6 AM local (sunrise time).
/// Verifies that:
/// - Hourly graph includes sunrise hour
/// - Wind speed conversion (knots to km/h) is accurate
/// - Current conditions reflect early morning state
#[test]
fn snapshot_bom_early_morning() {
    assert!(
        CONFIG.debugging.disable_weather_api_requests,
        "Test requires RUN_MODE=test environment variable"
    );

    if format!("{}", CONFIG.api.provider).to_lowercase() != "bom" {
        eprintln!("Skipping BOM early morning test - provider is set to '{}'", CONFIG.api.provider);
        return;
    }

    // 19:00 UTC = 6:00 AM Melbourne
    let clock = FixedClock::from_rfc3339("2025-10-25T19:00:00Z")
        .expect("Failed to create fixed clock");

    let result = generate_weather_dashboard_with_clock(&clock);
    assert!(result.is_ok(), "Dashboard generation failed: {:?}", result.err());

    let svg_content = fs::read_to_string(CONFIG.misc.generated_svg_name.clone())
        .expect("Failed to read generated SVG file");

    assert!(!svg_content.is_empty() && svg_content.contains("<svg"));
    insta::assert_snapshot!("bom_early_morning", svg_content);
}
