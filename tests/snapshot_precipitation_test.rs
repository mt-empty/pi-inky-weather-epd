//! Snapshot tests for precipitation pattern rendering
//!
//! These tests verify that the hourly graph correctly renders precipitation blocks
//! with the right SVG fill pattern (`rain` or `snow`) and the right `fill-opacity`
//! based on the chance threshold (< 50% → 25%, ≥ 50% → 35%).
//!
//! ## Opacity rules
//!
//! | Condition              | Pattern      | fill-opacity |
//! |------------------------|--------------|--------------|
//! | Rain, chance < 50%     | `url(#rain)` | 25%          |
//! | Rain, chance ≥ 50%     | `url(#rain)` | 35%          |
//! | Snow (any chance)      | `url(#snow)` | 35%          |
//!
//! ## Running
//!
//! ```bash
//! RUN_MODE=test cargo test --test snapshot_precipitation_test
//! ```

mod helpers;

use helpers::wiremock_setup;
use pi_inky_weather_epd::{clock::FixedClock, generate_weather_dashboard_injection, CONFIG};
use std::fs;
use std::path::Path;

/// Count occurrences of `fill="url(#<pattern>)" fill-opacity="<opacity>"` pairs in SVG.
/// Matches the exact attribute order produced by `generate_precipitation_pattern_svg`.
fn count_pattern_opacity(svg: &str, pattern: &str, opacity: &str) -> usize {
    let needle = format!(r#"fill="url(#{})" fill-opacity="{}""#, pattern, opacity);
    svg.matches(&needle).count()
}

/// Snapshot test: snowy conditions in interior Alaska (mid-day)
///
/// **Fixed Time**: Jan 15, 2026, 21:00:00 UTC = Jan 15, 2026, 12:00 PM AKST
///
/// **Fixture design** (`America/Anchorage`, GMT stored, hours 18–47 are snowy):
/// - All 24 hours in the forecast window: `snowfall=5cm`, `precipitation=2mm`, chance=85%
/// - `is_primarily_snow()` = true for all (`5×1.43=7.15 > 2.0×0.6=1.2`)
///
/// **What This Tests**:
/// - All 24 blocks render `url(#snow)` (not `url(#rain)`)
/// - All snow blocks are at 35% opacity (chance=85% ≥ 50%)
/// - No 25% opacity blocks are present (no low-chance rain hours)
/// - `{snow_colour}` template variable is substituted
#[tokio::test]
#[serial_test::serial]
async fn snapshot_open_meteo_alaska_snow() {
    if format!("{}", CONFIG.api.provider).to_lowercase() != "openmeteo" {
        eprintln!(
            "Skipping Alaska snow test - provider is set to '{}'",
            CONFIG.api.provider
        );
        return;
    }

    let mock_server = wiremock_setup::setup_open_meteo_mock(
        "tests/fixtures/alaska_snow/open_meteo_hourly_forecast.json",
        "tests/fixtures/alaska_snow/open_meteo_daily_forecast.json",
    )
    .await;
    std::env::set_var("OPEN_METEO_BASE_URL", mock_server.uri());

    let clock =
        FixedClock::from_rfc3339("2026-01-15T21:00:00Z").expect("Failed to create fixed clock");
    let output_svg_name = Path::new("tests/output/snapshot_open_meteo_alaska_snow.svg");

    let svg_content = tokio::task::spawn_blocking(move || {
        // RAII guard: restores TZ unconditionally, even if an assertion panics.
        struct TzGuard(Option<String>);
        impl Drop for TzGuard {
            fn drop(&mut self) {
                unsafe {
                    match self.0.take() {
                        Some(tz) => std::env::set_var("TZ", tz),
                        None => std::env::remove_var("TZ"),
                    }
                }
            }
        }
        let _tz_guard = TzGuard(std::env::var("TZ").ok());
        unsafe {
            std::env::set_var("TZ", "America/Anchorage");
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

        // All blocks must use the snow pattern
        assert!(
            svg.contains("url(#snow)"),
            "Expected url(#snow) in SVG output — is_primarily_snow() returned false for all hours."
        );
        assert!(
            !svg.contains("url(#rain)"),
            "Expected no url(#rain) in SVG output — all hours should be snow."
        );

        // All snow blocks must be at 35% opacity (chance=85% ≥ 50% threshold)
        let snow_35 = count_pattern_opacity(&svg, "snow", "35%");
        let snow_25 = count_pattern_opacity(&svg, "snow", "25%");
        assert_eq!(
            snow_25, 0,
            "Expected 0 snow blocks at 25% opacity (all have chance=85%), found {snow_25}"
        );
        assert_eq!(
            snow_35, 24,
            "Expected all 24 snow blocks at 35% opacity (all hours have chance=85%), found {snow_35}"
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

    std::env::remove_var("OPEN_METEO_BASE_URL");
    insta::assert_snapshot!(svg_content);
}

/// Snapshot test: mixed precipitation — rain at two opacity levels and snow in one 24h window
///
/// **Fixed Time**: Feb 1, 2026, 00:00:00 UTC (midnight UTC = start of the fixture window)
///
/// **Fixture design** (GMT timezone, hours 0–23):
/// - h00–h07: chance 10–45%, no snow   → `url(#rain)` at 25% opacity
/// - h08–h15: chance 50–90%, no snow   → `url(#rain)` at 35% opacity
/// - h16–h23: chance 85%, 5cm snow     → `url(#snow)` at 35% opacity
///
/// **What This Tests**:
/// - Both `url(#rain)` and `url(#snow)` patterns appear
/// - Low-chance rain blocks (h00–h07) render at 25% opacity
/// - High-chance rain blocks (h08–h15) render at 35% opacity
/// - Snow blocks (h16–h23) render at 35% opacity
/// - Exact block counts match the fixture (8 per category)
/// - No `url(#heavy-rain)` or other unexpected patterns appear
#[tokio::test]
#[serial_test::serial]
async fn snapshot_open_meteo_mixed_precip() {
    if format!("{}", CONFIG.api.provider).to_lowercase() != "openmeteo" {
        eprintln!(
            "Skipping mixed precip test - provider is set to '{}'",
            CONFIG.api.provider
        );
        return;
    }

    let mock_server = wiremock_setup::setup_open_meteo_mock(
        "tests/fixtures/mixed_precip/open_meteo_hourly_forecast.json",
        "tests/fixtures/mixed_precip/open_meteo_daily_forecast.json",
    )
    .await;
    std::env::set_var("OPEN_METEO_BASE_URL", mock_server.uri());

    let clock =
        FixedClock::from_rfc3339("2026-02-01T00:00:00Z").expect("Failed to create fixed clock");
    let output_svg_name = Path::new("tests/output/snapshot_open_meteo_mixed_precip.svg");

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

        // Both pattern types must be present
        assert!(
            svg.contains("url(#rain)"),
            "Expected url(#rain) in SVG output but it was not found."
        );
        assert!(
            svg.contains("url(#snow)"),
            "Expected url(#snow) in SVG output but it was not found."
        );

        // No unexpected patterns
        assert!(
            !svg.contains("url(#heavy-rain)"),
            "Unexpected url(#heavy-rain) found — heavy-rain pattern was removed."
        );

        // h00–h07: 8 rain blocks at 25% opacity (chance 10–45%, all < 50%)
        let rain_25 = count_pattern_opacity(&svg, "rain", "25%");
        assert_eq!(
            rain_25, 8,
            "Expected 8 rain blocks at 25% opacity (h00–h07, chance < 50%), found {rain_25}"
        );

        // h08–h15: 8 rain blocks at 35% opacity (chance 50–90%, all ≥ 50%)
        let rain_35 = count_pattern_opacity(&svg, "rain", "35%");
        assert_eq!(
            rain_35, 8,
            "Expected 8 rain blocks at 35% opacity (h08–h15, chance ≥ 50%), found {rain_35}"
        );

        // h16–h23: 8 snow blocks at 35% opacity (chance=85% ≥ 50%)
        let snow_35 = count_pattern_opacity(&svg, "snow", "35%");
        assert_eq!(
            snow_35, 8,
            "Expected 8 snow blocks at 35% opacity (h16–h23), found {snow_35}"
        );

        // No snow blocks at 25% opacity
        let snow_25 = count_pattern_opacity(&svg, "snow", "25%");
        assert_eq!(
            snow_25, 0,
            "Expected 0 snow blocks at 25% opacity, found {snow_25}"
        );

        // Template variable must be substituted
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

    std::env::remove_var("OPEN_METEO_BASE_URL");
    insta::assert_snapshot!(svg_content);
}
