//! Snapshot tests for Open-Meteo with `prefer_weather_codes = true`
//!
//! These tests are identical in structure to the core Open-Meteo snapshot tests
//! in `snapshot_provider_test.rs`, but run with WMO weather-code icon resolution
//! enabled, exercising the `src/domain/weather_code.rs` path instead of the
//! precipitation/cloud fallback path.
//!
//! ## Running These Tests
//!
//! `prefer_weather_codes` is a lazy-static config value, so the env override
//! must be present **before** the test binary starts.
//!
//! ```bash
//! RUN_MODE=test APP_RENDER_OPTIONS__PREFER_WEATHER_CODES=true \
//!   cargo test --test snapshot_open_meteo_prefer_codes_test
//! ```
//!
//! CI runs this automatically (see `.github/workflows/test.yml`).
//!
//! ## Reviewing Snapshots
//!
//! ```bash
//! RUN_MODE=test APP_RENDER_OPTIONS__PREFER_WEATHER_CODES=true \
//!   cargo test --test snapshot_open_meteo_prefer_codes_test
//! cargo insta review
//! ```

mod helpers;

use helpers::wiremock_setup;
use pi_inky_weather_epd::{clock::FixedClock, generate_weather_dashboard_injection, CONFIG};
use std::fs;
use std::path::Path;

/// Shared setup: start wiremock, generate the dashboard, return the SVG string.
///
/// Returns `None` when the test should be skipped (wrong provider or
/// `prefer_weather_codes` is not enabled).
///
/// The `insta::assert_snapshot!` call is intentionally left in each test
/// function so that insta can derive the snapshot name from the caller.
async fn run_prefer_codes_snapshot(
    time_rfc3339: &str,
    output_path: &'static str,
) -> Option<String> {
    if !CONFIG.render_options.prefer_weather_codes {
        eprintln!(
            "Skipping prefer-codes tests – set APP_RENDER_OPTIONS__PREFER_WEATHER_CODES=true \
             and re-run `cargo test --test snapshot_open_meteo_prefer_codes_test`"
        );
        return None;
    }
    if format!("{}", CONFIG.api.provider).to_lowercase() != "openmeteo" {
        eprintln!(
            "Skipping prefer-codes test – provider is '{}', expected 'openmeteo'",
            CONFIG.api.provider
        );
        return None;
    }

    let mock_server = wiremock_setup::setup_open_meteo_mock(
        "tests/fixtures/open_meteo_hourly_forecast.json",
        "tests/fixtures/open_meteo_daily_forecast.json",
    )
    .await;
    std::env::set_var("OPEN_METEO_BASE_URL", mock_server.uri());

    let clock = FixedClock::from_rfc3339(time_rfc3339).expect("invalid RFC3339 time");
    let output_svg_name = Path::new(output_path);

    let svg = tokio::task::spawn_blocking(move || {
        generate_weather_dashboard_injection(&clock, &CONFIG.misc.template_path, output_svg_name)
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

    std::env::remove_var("OPEN_METEO_BASE_URL");
    Some(svg)
}

// ---------------------------------------------------------------------------
// Tests – only the time, output path, and snapshot name differ
// ---------------------------------------------------------------------------

/// Oct 25 2025, 01:00 UTC = Oct 25 2025, 12:00 Melbourne (AEDT) – noon
#[tokio::test]
#[serial_test::serial]
async fn snapshot_open_meteo_dashboard_prefer_codes() {
    let Some(svg) = run_prefer_codes_snapshot(
        "2025-10-25T01:00:00Z",
        "tests/output/snapshot_open_meteo_dashboard_prefer_codes.svg",
    )
    .await
    else {
        return;
    };
    insta::assert_snapshot!(svg);
}

/// Oct 26 2025, 00:00 UTC = Oct 26 2025, 11:00 Melbourne (AEDT) – midnight boundary
#[tokio::test]
#[serial_test::serial]
async fn snapshot_open_meteo_midnight_boundary_prefer_codes() {
    let Some(svg) = run_prefer_codes_snapshot(
        "2025-10-26T00:00:00Z",
        "tests/output/snapshot_open_meteo_midnight_boundary_prefer_codes.svg",
    )
    .await
    else {
        return;
    };
    insta::assert_snapshot!(svg);
}

/// Oct 25 2025, 13:00 UTC = Oct 26 2025, 00:00 Melbourne (AEDT) – end of day
#[tokio::test]
#[serial_test::serial]
async fn snapshot_open_meteo_end_of_day_prefer_codes() {
    let Some(svg) = run_prefer_codes_snapshot(
        "2025-10-25T13:00:00Z",
        "tests/output/snapshot_open_meteo_end_of_day_prefer_codes.svg",
    )
    .await
    else {
        return;
    };
    insta::assert_snapshot!(svg);
}

/// Oct 25 2025, 16:00 UTC = Oct 26 2025, 03:00 Melbourne (AEDT) – early morning
#[tokio::test]
#[serial_test::serial]
async fn snapshot_open_meteo_early_morning_prefer_codes() {
    let Some(svg) = run_prefer_codes_snapshot(
        "2025-10-25T16:00:00Z",
        "tests/output/snapshot_open_meteo_early_morning_prefer_codes.svg",
    )
    .await
    else {
        return;
    };
    insta::assert_snapshot!(svg);
}
