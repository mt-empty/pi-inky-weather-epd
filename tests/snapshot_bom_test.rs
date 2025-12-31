//! BOM provider snapshot tests using fixtures
//!
//! These tests verify the complete dashboard generation pipeline with pre-cached fixture data.
//!
//! ## How These Tests Work
//!
//! 1. **Fixed Time**: Use FixedClock to ensure deterministic "current hour"
//! 2. **Fixture Data**: Load pre-defined weather data from cache (no HTTP calls)
//! 3. **Full Pipeline**: Run complete dashboard generation
//! 4. **Snapshot SVG**: Capture and compare the full SVG output
//!
//! ## Running These Tests
//!
//! ```bash
//! # Override provider to BOM and enable fixture loading
//! RUN_MODE=test APP_DEBUGGING__DISABLE_WEATHER_API_REQUESTS=true APP_API__PROVIDER=bom cargo test --test snapshot_bom_test
//! ```
//!
//! ## Reviewing Snapshots
//!
//! ```bash
//! RUN_MODE=test APP_DEBUGGING__DISABLE_WEATHER_API_REQUESTS=true APP_API__PROVIDER=bom cargo test --test snapshot_bom_test
//! cargo insta review  # Review and accept/reject changes
//! ```

mod helpers;

use helpers::test_utils;
use pi_inky_weather_epd::{
    clock::FixedClock, configs::settings::Providers, generate_weather_dashboard_injection, CONFIG,
};
use std::fs;

/// Common test logic for BOM snapshot tests
fn run_bom_snapshot_test(clock_time: &str, output_name: &str) -> String {
    // Skip if wrong provider
    if !test_utils::is_provider(Providers::Bom) {
        eprintln!(
            "Skipping BOM test - provider is set to '{}'",
            CONFIG.api.provider
        );
        return String::new();
    }

    let clock = FixedClock::from_rfc3339(clock_time)
        .unwrap_or_else(|_| panic!("Invalid clock time: {}", clock_time));

    let output_path = test_utils::outputs::bom(output_name);

    let result =
        generate_weather_dashboard_injection(&clock, &CONFIG.misc.template_path, &output_path);

    if let Err(e) = result {
        panic!("Dashboard generation failed: {e:?}");
    }

    fs::read_to_string(&output_path)
        .unwrap_or_else(|e| panic!("Failed to read SVG from {}: {e}", output_path.display()))
}

/// Test BOM provider dashboard generation
///
/// **Fixed Time**: Oct 25, 2025, 10:00 AM UTC = Oct 25, 2025, 9:00 PM Melbourne (AEDT)
///
/// **Fixture Files**:
/// - `bom_hourly_forecast.json` - Starts at 11:00 UTC
/// - `bom_daily_forecast.json` - Starts Oct 24
///
/// **Tests**: BOM API parsing, integer temps, knots→km/h conversion, Australian timezone
#[test]
fn snapshot_bom_dashboard() {
    let svg = run_bom_snapshot_test("2025-10-25T10:00:00Z", "dashboard");

    if !svg.is_empty() {
        insta::assert_snapshot!(svg);
    }
}

/// Test BOM at midnight boundary (date transition edge case)
///
/// **Fixed Time**: Oct 26, 2025, 00:00:00 UTC = Oct 26, 2025, 11:00 AM Melbourne (AEDT)
///
/// **Tests**: Midnight UTC boundary, BOM-specific date handling
#[test]
fn snapshot_bom_midnight_boundary() {
    let svg = run_bom_snapshot_test("2025-10-26T00:00:00Z", "midnight_boundary");

    if !svg.is_empty() {
        insta::assert_snapshot!(svg);
    }
}

/// Test BOM at local midnight (just after day rollover)
///
/// **Fixed Time**: Oct 25, 2025, 13:00:00 UTC = Oct 26, 2025, 12:00 AM Melbourne (AEDT)
///
/// **Tests**: Local midnight, daily forecast alignment with Australian calendar days
#[test]
fn snapshot_bom_local_midnight() {
    let svg = run_bom_snapshot_test("2025-10-25T13:00:00Z", "local_midnight");

    if !svg.is_empty() {
        insta::assert_snapshot!(svg);
    }
}

/// Test BOM during early morning hours
///
/// **Fixed Time**: Oct 25, 2025, 19:00:00 UTC = Oct 26, 2025, 6:00 AM Melbourne (AEDT)
///
/// **Tests**: Early morning rendering, hour labels, current conditions
#[test]
fn snapshot_bom_early_morning() {
    let svg = run_bom_snapshot_test("2025-10-25T19:00:00Z", "early_morning");

    if !svg.is_empty() {
        insta::assert_snapshot!(svg);
    }
}
