//! Provider-specific snapshot tests
//!
//! These tests verify that the complete dashboard generation pipeline produces
//! consistent output for each weather provider (BOM and Open-Meteo).
//!
//! ## How These Tests Work
//!
//! **Open-Meteo Tests** (using Wiremock):
//! 1. **Wiremock Server**: Start mock HTTP server with fixture data
//! 2. **URL Override**: Set OPEN_METEO_BASE_URL env var to point to mock server
//! 3. **Fixed Time**: Use FixedClock to ensure deterministic "current hour"
//! 4. **HTTP Calls**: Provider makes HTTP calls (intercepted by wiremock)
//! 5. **Snapshot SVG**: Capture and compare the full SVG output
//!
//! **BOM Tests** (using Fixtures):
//! 1. **Fixed Time**: Use FixedClock to ensure deterministic "current hour"
//! 2. **Fixture Data**: Load pre-defined weather data (no API calls)
//! 3. **Full Pipeline**: Run complete dashboard generation
//! 4. **Snapshot SVG**: Capture and compare the full SVG output
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
//!
//! ## Reviewing Snapshots
//!
//! On first run or after intentional changes:
//! ```bash
//! RUN_MODE=test APP_API__PROVIDER=open_meteo cargo test --test snapshot_provider_test
//! RUN_MODE=test APP_API__PROVIDER=bom cargo test --test snapshot_provider_test
//! cargo insta review  # Review and accept/reject changes
//! ```

mod helpers;

use helpers::wiremock_setup;
use pi_inky_weather_epd::{clock::FixedClock, generate_weather_dashboard_injection, CONFIG};
use std::fs;
use std::path::Path;

/// Test Open-Meteo provider dashboard generation with wiremock
///
/// **Fixed Time**: Oct 25, 2025, 1:00 AM UTC = Oct 25, 2025, 12:00 PM Melbourne (AEDT)
///
/// **Mocked Endpoint**:
/// - `GET /v1/forecast` → Returns `tests/fixtures/open_meteo_forecast.json`
///
/// **What This Tests**:
/// - Open-Meteo API response parsing (parallel array structure)
/// - Float temperature handling
/// - Global provider (lat/lon instead of geohash)
/// - GMT timezone conversion to Melbourne (AEDT)
/// - Dashboard rendering with Open-Meteo data structure
/// - HTTP client integration with wiremock
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
#[tokio::test]
#[serial_test::serial]
async fn snapshot_open_meteo_dashboard() {
    // Skip test if BOM provider is configured (test is provider-specific)
    if format!("{}", CONFIG.api.provider).to_lowercase() != "openmeteo" {
        eprintln!(
            "Skipping Open-Meteo test - provider is set to '{}'",
            CONFIG.api.provider
        );
        return;
    }

    // Start wiremock server with fixture data
    let mock_server = wiremock_setup::setup_open_meteo_mock(
        "tests/fixtures/open_meteo_hourly_forecast.json",
        "tests/fixtures/open_meteo_daily_forecast.json",
    )
    .await;

    // Override Open-Meteo base URL to point to mock server
    std::env::set_var("OPEN_METEO_BASE_URL", mock_server.uri());

    // Fixed time: Oct 25, 2025, 1:00 AM UTC
    // In Melbourne AEDT (UTC+11): Oct 25, 2025, 12:00 PM (noon)
    let clock =
        FixedClock::from_rfc3339("2025-10-25T01:00:00Z").expect("Failed to create fixed clock");

    let output_svg_name = Path::new("tests/output/snapshot_open_meteo_dashboard.svg");

    // Run sync dashboard generation in blocking task
    let svg_content = tokio::task::spawn_blocking(move || {
        let result = generate_weather_dashboard_injection(
            &clock,
            &CONFIG.misc.template_path,
            output_svg_name,
        );
        assert!(
            result.is_ok(),
            "Dashboard generation failed: {:?}",
            result.err()
        );

        let svg = fs::read_to_string(output_svg_name).expect("Failed to read generated SVG file");
        assert!(!svg.is_empty(), "Generated SVG should not be empty");
        assert!(svg.contains("<svg"), "Generated file should be valid SVG");
        svg
    })
    .await
    .expect("Task panicked");

    // Cleanup
    std::env::remove_var("OPEN_METEO_BASE_URL");

    // Snapshot the SVG structure
    insta::assert_snapshot!(svg_content);
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
#[tokio::test]
#[serial_test::serial]
async fn snapshot_open_meteo_midnight_boundary() {
    if format!("{}", CONFIG.api.provider).to_lowercase() != "openmeteo" {
        eprintln!(
            "Skipping Open-Meteo midnight test - provider is set to '{}'",
            CONFIG.api.provider
        );
        return;
    }

    let mock_server = wiremock_setup::setup_open_meteo_mock(
        "tests/fixtures/open_meteo_hourly_forecast.json",
        "tests/fixtures/open_meteo_daily_forecast.json",
    )
    .await;
    std::env::set_var("OPEN_METEO_BASE_URL", mock_server.uri());

    let clock =
        FixedClock::from_rfc3339("2025-10-26T00:00:00Z").expect("Failed to create fixed clock");
    let output_svg_name = Path::new("tests/output/snapshot_open_meteo_midnight_boundary.svg");

    let svg_content = tokio::task::spawn_blocking(move || {
        let result = generate_weather_dashboard_injection(
            &clock,
            &CONFIG.misc.template_path,
            output_svg_name,
        );
        assert!(
            result.is_ok(),
            "Dashboard generation failed: {:?}",
            result.err()
        );
        let svg = fs::read_to_string(output_svg_name).expect("Failed to read generated SVG file");
        assert!(!svg.is_empty() && svg.contains("<svg"));
        svg
    })
    .await
    .expect("Task panicked");

    std::env::remove_var("OPEN_METEO_BASE_URL");
    insta::assert_snapshot!(svg_content);
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
#[tokio::test]
#[serial_test::serial]
async fn snapshot_open_meteo_end_of_day() {
    if format!("{}", CONFIG.api.provider).to_lowercase() != "openmeteo" {
        eprintln!(
            "Skipping Open-Meteo end-of-day test - provider is set to '{}'",
            CONFIG.api.provider
        );
        return;
    }

    let mock_server = wiremock_setup::setup_open_meteo_mock(
        "tests/fixtures/open_meteo_hourly_forecast.json",
        "tests/fixtures/open_meteo_daily_forecast.json",
    )
    .await;
    std::env::set_var("OPEN_METEO_BASE_URL", mock_server.uri());

    let clock =
        FixedClock::from_rfc3339("2025-10-25T13:00:00Z").expect("Failed to create fixed clock");
    let output_svg_name = Path::new("tests/output/snapshot_open_meteo_end_of_day.svg");

    let svg_content = tokio::task::spawn_blocking(move || {
        let result = generate_weather_dashboard_injection(
            &clock,
            &CONFIG.misc.template_path,
            output_svg_name,
        );
        assert!(
            result.is_ok(),
            "Dashboard generation failed: {:?}",
            result.err()
        );
        let svg = fs::read_to_string(output_svg_name).expect("Failed to read generated SVG file");
        assert!(!svg.is_empty() && svg.contains("<svg"));
        svg
    })
    .await
    .expect("Task panicked");

    std::env::remove_var("OPEN_METEO_BASE_URL");
    insta::assert_snapshot!(svg_content);
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
#[tokio::test]
#[serial_test::serial]
async fn snapshot_open_meteo_early_morning() {
    if format!("{}", CONFIG.api.provider).to_lowercase() != "openmeteo" {
        eprintln!(
            "Skipping Open-Meteo early morning test - provider is set to '{}'",
            CONFIG.api.provider
        );
        return;
    }

    let mock_server = wiremock_setup::setup_open_meteo_mock(
        "tests/fixtures/open_meteo_hourly_forecast.json",
        "tests/fixtures/open_meteo_daily_forecast.json",
    )
    .await;
    std::env::set_var("OPEN_METEO_BASE_URL", mock_server.uri());

    let clock =
        FixedClock::from_rfc3339("2025-10-25T16:00:00Z").expect("Failed to create fixed clock");
    let output_svg_name = Path::new("tests/output/snapshot_open_meteo_early_morning.svg");

    let svg_content = tokio::task::spawn_blocking(move || {
        let result = generate_weather_dashboard_injection(
            &clock,
            &CONFIG.misc.template_path,
            output_svg_name,
        );
        assert!(
            result.is_ok(),
            "Dashboard generation failed: {:?}",
            result.err()
        );
        let svg = fs::read_to_string(output_svg_name).expect("Failed to read generated SVG file");
        assert!(!svg.is_empty() && svg.contains("<svg"));
        svg
    })
    .await
    .expect("Task panicked");

    std::env::remove_var("OPEN_METEO_BASE_URL");
    insta::assert_snapshot!(svg_content);
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
#[tokio::test]
#[serial_test::serial]
async fn snapshot_bom_dashboard() {
    // Skip test if Open-Meteo provider is configured (test is provider-specific)
    if format!("{}", CONFIG.api.provider).to_lowercase() != "bom" {
        eprintln!(
            "Skipping BOM test - provider is set to '{}'",
            CONFIG.api.provider
        );
        return;
    }

    // Start wiremock server with BOM fixture data
    let mock_server = wiremock_setup::setup_bom_mock(
        "tests/fixtures/bom_daily_forecast.json",
        "tests/fixtures/bom_hourly_forecast.json",
    )
    .await;

    // Override BOM base URL to point to mock server
    // Note: BOM URLs are constructed as {base_url}/{geohash}/forecasts/{frequency}
    std::env::set_var(
        "BOM_BASE_URL",
        format!("{}/v1/locations", mock_server.uri()),
    );

    // Fixed time: Oct 25, 2025, 10:00 AM UTC
    // In Melbourne AEDT (UTC+11): Oct 25, 2025, 9:00 PM
    // This is just before the first hourly forecast (11:00 UTC) in the fixture
    let clock =
        FixedClock::from_rfc3339("2025-10-25T10:00:00Z").expect("Failed to create fixed clock");

    let output_svg_name = Path::new("tests/output/snapshot_bom_dashboard.svg");

    // Run sync dashboard generation in blocking task
    let svg_content = tokio::task::spawn_blocking(move || {
        let result = generate_weather_dashboard_injection(
            &clock,
            &CONFIG.misc.template_path,
            output_svg_name,
        );
        assert!(
            result.is_ok(),
            "Dashboard generation failed: {:?}",
            result.err()
        );

        let svg = fs::read_to_string(output_svg_name).expect("Failed to read generated SVG file");
        assert!(!svg.is_empty(), "Generated SVG should not be empty");
        assert!(svg.contains("<svg"), "Generated file should be valid SVG");
        svg
    })
    .await
    .expect("Task panicked");

    // Cleanup
    std::env::remove_var("BOM_BASE_URL");

    // Snapshot the SVG structure
    insta::assert_snapshot!(svg_content);
}

/// Test BOM at midnight boundary (date transition edge case)
///
/// **Fixed Time**: Oct 26, 2025, 00:00:00 UTC = Oct 26, 2025, 11:00 AM Melbourne (AEDT)
///
/// **Edge Case**: Midnight UTC boundary for BOM provider.
/// Verifies BOM-specific parsing handles date transitions correctly.
#[tokio::test]
#[serial_test::serial]
async fn snapshot_bom_midnight_boundary() {
    if format!("{}", CONFIG.api.provider).to_lowercase() != "bom" {
        eprintln!(
            "Skipping BOM midnight test - provider is set to '{}'",
            CONFIG.api.provider
        );
        return;
    }

    let mock_server = wiremock_setup::setup_bom_mock(
        "tests/fixtures/bom_daily_forecast.json",
        "tests/fixtures/bom_hourly_forecast.json",
    )
    .await;
    std::env::set_var(
        "BOM_BASE_URL",
        format!("{}/v1/locations", mock_server.uri()),
    );

    // Midnight UTC on Oct 26 = 11:00 AM Melbourne
    let clock =
        FixedClock::from_rfc3339("2025-10-26T00:00:00Z").expect("Failed to create fixed clock");

    let output_svg_name = Path::new("tests/output/snapshot_bom_midnight_boundary.svg");

    let svg_content = tokio::task::spawn_blocking(move || {
        let result = generate_weather_dashboard_injection(
            &clock,
            &CONFIG.misc.template_path,
            output_svg_name,
        );
        assert!(
            result.is_ok(),
            "Dashboard generation failed: {:?}",
            result.err()
        );
        let svg = fs::read_to_string(output_svg_name).expect("Failed to read generated SVG file");
        assert!(!svg.is_empty() && svg.contains("<svg"));
        svg
    })
    .await
    .expect("Task panicked");

    std::env::remove_var("BOM_BASE_URL");
    insta::assert_snapshot!(svg_content);
}

/// Test BOM at local midnight (just after day rollover)
///
/// **Fixed Time**: Oct 25, 2025, 13:00:00 UTC = Oct 26, 2025, 12:00 AM Melbourne (AEDT)
///
/// **Edge Case**: Tests BOM provider at local midnight.
/// Verifies daily forecast alignment with Australian calendar days.
#[tokio::test]
#[serial_test::serial]
async fn snapshot_bom_local_midnight() {
    if format!("{}", CONFIG.api.provider).to_lowercase() != "bom" {
        eprintln!(
            "Skipping BOM local midnight test - provider is set to '{}'",
            CONFIG.api.provider
        );
        return;
    }

    let mock_server = wiremock_setup::setup_bom_mock(
        "tests/fixtures/bom_daily_forecast.json",
        "tests/fixtures/bom_hourly_forecast.json",
    )
    .await;
    std::env::set_var(
        "BOM_BASE_URL",
        format!("{}/v1/locations", mock_server.uri()),
    );

    // 13:00 UTC = Midnight Melbourne (Oct 26)
    let clock =
        FixedClock::from_rfc3339("2025-10-25T13:00:00Z").expect("Failed to create fixed clock");

    let output_svg_name = Path::new("tests/output/snapshot_bom_local_midnight.svg");

    let svg_content = tokio::task::spawn_blocking(move || {
        let result = generate_weather_dashboard_injection(
            &clock,
            &CONFIG.misc.template_path,
            output_svg_name,
        );
        assert!(
            result.is_ok(),
            "Dashboard generation failed: {:?}",
            result.err()
        );
        let svg = fs::read_to_string(output_svg_name).expect("Failed to read generated SVG file");
        assert!(!svg.is_empty() && svg.contains("<svg"));
        svg
    })
    .await
    .expect("Task panicked");

    std::env::remove_var("BOM_BASE_URL");
    insta::assert_snapshot!(svg_content);
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
#[tokio::test]
#[serial_test::serial]
async fn snapshot_bom_early_morning() {
    if format!("{}", CONFIG.api.provider).to_lowercase() != "bom" {
        eprintln!(
            "Skipping BOM early morning test - provider is set to '{}'",
            CONFIG.api.provider
        );
        return;
    }

    let mock_server = wiremock_setup::setup_bom_mock(
        "tests/fixtures/bom_daily_forecast.json",
        "tests/fixtures/bom_hourly_forecast.json",
    )
    .await;
    std::env::set_var(
        "BOM_BASE_URL",
        format!("{}/v1/locations", mock_server.uri()),
    );

    // 19:00 UTC = 6:00 AM Melbourne
    let clock =
        FixedClock::from_rfc3339("2025-10-25T19:00:00Z").expect("Failed to create fixed clock");

    let output_svg_name = Path::new("tests/output/snapshot_bom_early_morning.svg");

    let svg_content = tokio::task::spawn_blocking(move || {
        let result = generate_weather_dashboard_injection(
            &clock,
            &CONFIG.misc.template_path,
            output_svg_name,
        );
        assert!(
            result.is_ok(),
            "Dashboard generation failed: {:?}",
            result.err()
        );
        let svg = fs::read_to_string(output_svg_name).expect("Failed to read generated SVG file");
        assert!(!svg.is_empty() && svg.contains("<svg"));
        svg
    })
    .await
    .expect("Task panicked");

    std::env::remove_var("BOM_BASE_URL");
    insta::assert_snapshot!(svg_content);
}

/// Test Open-Meteo GMT boundary in NY timezone (6pm local)
///
/// **Fixed Time**: Dec 28, 2025, 23:00:00 UTC = Dec 28, 2025, 6:00 PM NY (EST)
/// **Timezone**: America/New_York (EST, UTC-5)
/// **Fixture**: `ny_6pm_before_gmt/open_meteo_forecast.json` (23:00 UTC is still Dec 28 in UTC, 6pm Dec 28 in NY)
///
/// **Edge Case**: Tests dashboard generation at 6pm local (NY) which is 23:00 UTC (still Dec 28).
/// This is 1 hour before the UTC midnight boundary. The daily forecast should show days
/// aligned with the local (NY) calendar.
#[tokio::test]
#[serial_test::serial]
async fn snapshot_open_meteo_ny_6pm_before_gmt_boundary() {
    if format!("{}", CONFIG.api.provider).to_lowercase() != "openmeteo" {
        eprintln!(
            "Skipping Open-Meteo NY 6pm test - provider is set to '{}'",
            CONFIG.api.provider
        );
        return;
    }

    let mock_server = wiremock_setup::setup_open_meteo_mock(
        "tests/fixtures/ny_6pm_before_gmt/open_meteo_hourly_forecast.json",
        "tests/fixtures/ny_6pm_before_gmt/open_meteo_daily_forecast.json",
    )
    .await;
    std::env::set_var("OPEN_METEO_BASE_URL", mock_server.uri());

    let clock =
        FixedClock::from_rfc3339("2025-12-28T23:00:00Z").expect("Failed to create fixed clock");
    let output_svg_name =
        Path::new("tests/output/snapshot_open_meteo_ny_6pm_before_gmt_boundary.svg");

    let svg_content = tokio::task::spawn_blocking(move || {
        // Set timezone
        let original_tz = std::env::var("TZ").ok();
        unsafe {
            std::env::set_var("TZ", "America/New_York");
        }

        let result = generate_weather_dashboard_injection(
            &clock,
            &CONFIG.misc.template_path,
            output_svg_name,
        );
        assert!(
            result.is_ok(),
            "Dashboard generation failed: {:?}",
            result.err()
        );
        let svg = fs::read_to_string(output_svg_name).expect("Failed to read generated SVG file");
        assert!(!svg.is_empty() && svg.contains("<svg"));

        // Restore timezone
        unsafe {
            match original_tz {
                Some(tz) => std::env::set_var("TZ", tz),
                None => std::env::remove_var("TZ"),
            }
        }
        svg
    })
    .await
    .expect("Task panicked");

    std::env::remove_var("OPEN_METEO_BASE_URL");
    insta::assert_snapshot!(svg_content);
}

/// Test Open-Meteo GMT boundary in NY timezone (7pm local)
///
/// **Fixed Time**: Dec 29, 2025, 00:00:00 UTC = Dec 28, 2025, 7:00 PM NY (EST)
/// **Timezone**: America/New_York (EST, UTC-5)
/// **Fixture**: `ny_7pm_after_gmt/open_meteo_forecast.json` (00:00 UTC is Dec 29 in UTC, but still 7pm Dec 28 in NY)
///
/// **Edge Case**: Tests dashboard generation at 7pm local (NY) which is 00:00 UTC (Dec 29).
/// Even though UTC has rolled over to Dec 29, it's still Dec 28 locally in NY.
///
/// **Expected Behavior**:
/// - Fixture dates start from Dec 29 (missing Dec 28) because API doesn't include yesterday
/// - Dashboard shows "Incomplete Data" warning (today's data missing)
/// - Sunrise/sunset show "NA" (today's astronomical data missing)
/// - First forecast card shows "Mon" (Dec 29, tomorrow) - this HAS data and is correct
/// - Dashboard displays tomorrow through +6 days (6 cards), not today
/// - Only today (Day 0) is used for sunrise/sunset, which are missing
///
/// **Note**: To fix the incomplete data warning, API request needs `past_days=1` parameter
/// to include Dec 28 in the fixture.
#[tokio::test]
#[serial_test::serial]
async fn snapshot_open_meteo_ny_7pm_after_gmt_boundary() {
    if format!("{}", CONFIG.api.provider).to_lowercase() != "openmeteo" {
        eprintln!(
            "Skipping Open-Meteo NY 7pm test - provider is set to '{}'",
            CONFIG.api.provider
        );
        return;
    }

    let mock_server = wiremock_setup::setup_open_meteo_mock(
        "tests/fixtures/ny_7pm_after_gmt/open_meteo_hourly_forecast.json",
        "tests/fixtures/ny_7pm_after_gmt/open_meteo_daily_forecast.json",
    )
    .await;
    std::env::set_var("OPEN_METEO_BASE_URL", mock_server.uri());

    let clock =
        FixedClock::from_rfc3339("2025-12-29T00:00:00Z").expect("Failed to create fixed clock");
    let output_svg_name =
        Path::new("tests/output/snapshot_open_meteo_ny_7pm_after_gmt_boundary.svg");

    let svg_content = tokio::task::spawn_blocking(move || {
        // Set timezone
        let original_tz = std::env::var("TZ").ok();
        unsafe {
            std::env::set_var("TZ", "America/New_York");
        }

        let result = generate_weather_dashboard_injection(
            &clock,
            &CONFIG.misc.template_path,
            output_svg_name,
        );
        assert!(
            result.is_ok(),
            "Dashboard generation failed: {:?}",
            result.err()
        );
        let svg = fs::read_to_string(output_svg_name).expect("Failed to read generated SVG file");
        assert!(!svg.is_empty() && svg.contains("<svg"));

        // Restore timezone
        unsafe {
            match original_tz {
                Some(tz) => std::env::set_var("TZ", tz),
                None => std::env::remove_var("TZ"),
            }
        }
        svg
    })
    .await
    .expect("Task panicked");

    std::env::remove_var("OPEN_METEO_BASE_URL");
    insta::assert_snapshot!(svg_content);
}
