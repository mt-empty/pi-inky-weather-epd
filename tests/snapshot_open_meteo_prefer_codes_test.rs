//! Snapshot tests for Open-Meteo with `prefer_weather_codes = true`
//!
//! These tests are identical in structure to the core Open-Meteo snapshot tests
//! in `snapshot_provider_test.rs`, but run with WMO weather-code icon resolution
//! enabled, exercising the `src/domain/weather_code.rs` path instead of the
//! precipitation/cloud fallback path.
//!
//! ## Running These Tests
//!
//! ```bash
//! cargo test --test snapshot_open_meteo_prefer_codes_test
//! ```
//!
//! ## Reviewing Snapshots
//!
//! ```bash
//! cargo test --test snapshot_open_meteo_prefer_codes_test
//! cargo insta review
//! ```

mod helpers;

use helpers::test_utils;
use helpers::wiremock_setup;
use pi_inky_weather_epd::configs::settings::Providers;
use pi_inky_weather_epd::{clock::FixedClock, generate_weather_dashboard_injection};
use std::fs;
use std::path::Path;

/// Shared setup: start wiremock, install a prefer-weather-codes config,
/// generate the dashboard, and return the SVG string.
///
/// The `insta::assert_snapshot!` call is intentionally left in each test
/// function so that insta can derive the snapshot name from the caller.
async fn run_prefer_codes_snapshot(time_rfc3339: &str, output_path: &'static str) -> String {
    let mock_server = wiremock_setup::setup_open_meteo_mock(
        "tests/fixtures/open_meteo_hourly_forecast.json",
        "tests/fixtures/open_meteo_daily_forecast.json",
    )
    .await;
    let mock_base_url = url::Url::parse(&mock_server.uri()).expect("invalid mock server URL");
    let settings = test_utils::test_settings(|settings| {
        settings.api.provider = Providers::OpenMeteo;
        settings.api.open_meteo_base_url = mock_base_url;
        settings.render_options.prefer_weather_codes = true;
    });

    let clock = FixedClock::from_rfc3339(time_rfc3339).expect("invalid RFC3339 time");
    let output_svg_name = Path::new(output_path);

    let svg = tokio::task::spawn_blocking(move || {
        generate_weather_dashboard_injection(&settings, &clock, output_svg_name)
            .expect("dashboard generation failed");
        let svg = fs::read_to_string(output_svg_name).expect("failed to read generated SVG");
        assert!(
            !svg.is_empty() && svg.contains("<svg"),
            "generated file is not valid SVG"
        );
        svg
    })
    .await
    .expect("task panicked");

    svg
}

// ---------------------------------------------------------------------------
// Tests – only the time, output path, and snapshot name differ
// ---------------------------------------------------------------------------

/// Oct 25 2025, 01:00 UTC = Oct 25 2025, 12:00 Melbourne (AEDT) – noon
#[tokio::test]
async fn snapshot_open_meteo_dashboard_prefer_codes() {
    let svg = run_prefer_codes_snapshot(
        "2025-10-25T01:00:00Z",
        "tests/output/snapshot_open_meteo_dashboard_prefer_codes.svg",
    )
    .await;
    insta::assert_snapshot!(svg);
}

/// Oct 26 2025, 00:00 UTC = Oct 26 2025, 11:00 Melbourne (AEDT) – midnight boundary
#[tokio::test]
async fn snapshot_open_meteo_midnight_boundary_prefer_codes() {
    let svg = run_prefer_codes_snapshot(
        "2025-10-26T00:00:00Z",
        "tests/output/snapshot_open_meteo_midnight_boundary_prefer_codes.svg",
    )
    .await;
    insta::assert_snapshot!(svg);
}

/// Oct 25 2025, 13:00 UTC = Oct 26 2025, 00:00 Melbourne (AEDT) – end of day
#[tokio::test]
async fn snapshot_open_meteo_end_of_day_prefer_codes() {
    let svg = run_prefer_codes_snapshot(
        "2025-10-25T13:00:00Z",
        "tests/output/snapshot_open_meteo_end_of_day_prefer_codes.svg",
    )
    .await;
    insta::assert_snapshot!(svg);
}

/// Oct 25 2025, 16:00 UTC = Oct 26 2025, 03:00 Melbourne (AEDT) – early morning
#[tokio::test]
async fn snapshot_open_meteo_early_morning_prefer_codes() {
    let svg = run_prefer_codes_snapshot(
        "2025-10-25T16:00:00Z",
        "tests/output/snapshot_open_meteo_early_morning_prefer_codes.svg",
    )
    .await;
    insta::assert_snapshot!(svg);
}
