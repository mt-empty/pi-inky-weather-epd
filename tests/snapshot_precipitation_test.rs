//! Snapshot tests for precipitation glyph rendering
//!
//! These tests verify that the hourly graph correctly renders precipitation using the
//! unified SVG approach: inline glyphs (circles for snow, capsule paths for rain) placed
//! via LCG, clipped to a trapezoid per block, and filled with a per-block linear gradient.
//!
//! ## Glyph signatures
//!
//! | Type | SVG element                          | fill-opacity |
//! |------|--------------------------------------|--------------|
//! | Snow | `<circle … fill-opacity="0.85"/>`    | 0.85         |
//! | Rain | `<path … fill-opacity="0.8" …/>`     | 0.8          |
//!
//! ## Running
//!
//! ```bash
//! cargo test --test snapshot_precipitation_test
//! ```

mod helpers;

use helpers::test_utils;
use helpers::wiremock_setup;
use pi_inky_weather_epd::{clock::FixedClock, generate_weather_dashboard_injection};
use std::fs;
use std::path::Path;

/// Snapshot test: snowy conditions in interior Alaska (mid-day)
///
/// **Fixed Time**: Jan 15, 2026, 21:00:00 UTC = Jan 15, 2026, 12:00 PM AKST
///
/// **Fixture design** (`America/Anchorage`, hours 18–47 UTC are snowy):
/// - All 24 hours in the forecast window: `snowfall=5cm`, `precipitation=2mm`, chance=85%
/// - `is_primarily_snow()` = true for all (`5×1.43=7.15 > 2.0×0.6=1.2`)
///
/// **What This Tests**:
/// - Snow circle glyphs (`fill-opacity="0.85"`) appear in the graph
/// - Rain drop glyphs (`fill-opacity="0.8"`) are absent
/// - `{snow_colour}` template variable is substituted
#[tokio::test]
async fn snapshot_open_meteo_alaska_snow() {
    let mock_server = wiremock_setup::setup_open_meteo_mock(
        "tests/fixtures/alaska_snow/open_meteo_hourly_forecast.json",
        "tests/fixtures/alaska_snow/open_meteo_daily_forecast.json",
    )
    .await;
    let settings =
        test_utils::open_meteo_settings_in_tz(&mock_server.uri(), chrono_tz::America::Anchorage);

    let clock =
        FixedClock::from_rfc3339("2026-01-15T21:00:00Z").expect("Failed to create fixed clock");
    let output_svg_name = Path::new("tests/output/snapshot_open_meteo_alaska_snow.svg");

    let svg_content = tokio::task::spawn_blocking(move || {
        let result = generate_weather_dashboard_injection(
            &settings,
            &clock,
            output_svg_name,
        );
        assert!(
            result.is_ok(),
            "Dashboard generation failed: {:?}",
            result.err()
        );

        let svg = fs::read_to_string(output_svg_name).expect("Failed to read generated SVG file");
        assert!(!svg.is_empty() && svg.contains("<svg"));

        // Snow hours must produce circle glyphs (fill-opacity="0.85")
        assert!(
            svg.contains(r#"fill-opacity="0.85""#),
            "Expected snow circle glyphs (fill-opacity=\"0.85\") in SVG — is_primarily_snow() may have returned false for all hours."
        );

        // No rain drop glyphs (fill-opacity="0.8") should appear for an all-snow forecast
        assert!(
            !svg.contains(r#"fill-opacity="0.8""#),
            "Unexpected rain drop glyphs (fill-opacity=\"0.8\") in SVG — all hours should be snow."
        );

        // Template variable must be substituted
        assert!(
            !svg.contains("{snow_colour}"),
            "Template variable {{snow_colour}} was not substituted in the SVG output."
        );

        svg
    })
    .await
    .expect("Task panicked");

    insta::assert_snapshot!(svg_content);
}

/// Snapshot test: mixed precipitation — rain and snow in one 24h window
///
/// **Fixed Time**: Feb 1, 2026, 00:00:00 UTC (midnight UTC = start of the fixture window)
///
/// **Fixture design** (GMT timezone, hours 0–23):
/// - h00–h07: chance 10–45%, no snow   → rain drop glyphs, low-opacity gradient
/// - h08–h15: chance 50–90%, no snow   → rain drop glyphs, high-opacity gradient
/// - h16–h23: chance 85%, 5cm snow     → snow circle glyphs
///
/// **What This Tests**:
/// - Both snow circles (`fill-opacity="0.85"`) and rain drops (`fill-opacity="0.8"`) appear
/// - No `url(#heavy-rain)` or other unexpected legacy patterns appear
/// - Template variables are substituted
#[tokio::test]
async fn snapshot_open_meteo_mixed_precip() {
    let mock_server = wiremock_setup::setup_open_meteo_mock(
        "tests/fixtures/mixed_precip/open_meteo_hourly_forecast.json",
        "tests/fixtures/mixed_precip/open_meteo_daily_forecast.json",
    )
    .await;
    let settings = test_utils::open_meteo_settings(&mock_server.uri());

    let clock =
        FixedClock::from_rfc3339("2026-02-01T00:00:00Z").expect("Failed to create fixed clock");
    let output_svg_name = Path::new("tests/output/snapshot_open_meteo_mixed_precip.svg");

    let svg_content = tokio::task::spawn_blocking(move || {
        let result = generate_weather_dashboard_injection(&settings, &clock, output_svg_name);
        assert!(
            result.is_ok(),
            "Dashboard generation failed: {:?}",
            result.err()
        );

        let svg = fs::read_to_string(output_svg_name).expect("Failed to read generated SVG file");
        assert!(!svg.is_empty() && svg.contains("<svg"));

        // Rain hours must produce drop glyphs
        assert!(
            svg.contains(r#"fill-opacity="0.8""#),
            "Expected rain drop glyphs (fill-opacity=\"0.8\") in SVG output but none found."
        );

        // Snow hours must produce circle glyphs
        assert!(
            svg.contains(r#"fill-opacity="0.85""#),
            "Expected snow circle glyphs (fill-opacity=\"0.85\") in SVG output but none found."
        );

        // No legacy heavy-rain pattern reference
        assert!(
            !svg.contains("url(#heavy-rain)"),
            "Unexpected url(#heavy-rain) found — heavy-rain pattern was removed."
        );

        // Template variables must be substituted
        assert!(
            !svg.contains("{snow_colour}"),
            "Template variable {{snow_colour}} was not substituted in the SVG output."
        );
        assert!(
            !svg.contains("{rain_colour}"),
            "Template variable {{rain_colour}} was not substituted in the SVG output."
        );

        svg
    })
    .await
    .expect("Task panicked");

    insta::assert_snapshot!(svg_content);
}
