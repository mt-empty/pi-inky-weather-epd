//! Open-Meteo provider snapshot tests using Wiremock
//!
//! These tests verify the complete dashboard generation pipeline with mocked HTTP responses.
//!
//! ## How These Tests Work
//!
//! 1. **Wiremock Server**: Start mock HTTP server with fixture data
//! 2. **URL Override**: Set OPEN_METEO_BASE_URL env var to point to mock server
//! 3. **Fixed Time**: Use FixedClock to ensure deterministic "current hour"
//! 4. **HTTP Calls**: Provider makes HTTP calls (intercepted by wiremock)
//! 5. **Snapshot SVG**: Capture and compare the full SVG output
//!
//! ## Running These Tests
//!
//! ```bash
//! # Default (Open-Meteo is default provider in test.toml)
//! RUN_MODE=test cargo test --test snapshot_open_meteo_test
//!
//! # Explicit provider override
//! RUN_MODE=test APP_API__PROVIDER=open_meteo cargo test --test snapshot_open_meteo_test
//! ```
//!
//! ## Reviewing Snapshots
//!
//! ```bash
//! RUN_MODE=test cargo test --test snapshot_open_meteo_test
//! cargo insta review  # Review and accept/reject changes
//! ```

mod helpers;

use helpers::{test_utils, wiremock_setup};
use pi_inky_weather_epd::{
    clock::FixedClock, configs::settings::Providers, generate_weather_dashboard_injection, CONFIG,
};
use std::fs;
use test_utils::EnvVarGuard;

/// Configuration for an Open-Meteo snapshot test
struct TestCase {
    fixture_path: &'static str,
    clock_time: &'static str,
    timezone: Option<&'static str>,
    output_name: &'static str,
}

/// Common test logic for Open-Meteo snapshot tests
async fn run_open_meteo_snapshot_test(config: TestCase) -> String {
    // Skip if wrong provider
    if !test_utils::is_provider(Providers::OpenMeteo) {
        eprintln!(
            "Skipping Open-Meteo test - provider is set to '{}'",
            CONFIG.api.provider
        );
        return String::new();
    }

    // Setup wiremock server
    let mock_server = wiremock_setup::setup_open_meteo_mock(config.fixture_path).await;
    let _url_guard = EnvVarGuard::new("OPEN_METEO_BASE_URL", &mock_server.uri());
    let _tz_guard = config.timezone.map(|tz| EnvVarGuard::new("TZ", tz));

    // Create fixed clock
    let clock = FixedClock::from_rfc3339(config.clock_time)
        .unwrap_or_else(|_| panic!("Invalid clock time: {}", config.clock_time));

    let output_path = test_utils::outputs::open_meteo(config.output_name);

    // Run dashboard generation in blocking task
    tokio::task::spawn_blocking(move || {
        let result =
            generate_weather_dashboard_injection(&clock, &CONFIG.misc.template_path, &output_path);

        if let Err(e) = result {
            panic!("Dashboard generation failed: {e:?}");
        }

        fs::read_to_string(&output_path)
            .unwrap_or_else(|e| panic!("Failed to read SVG from {}: {e}", output_path.display()))
    })
    .await
    .expect("Task panicked")
}

/// Test Open-Meteo provider dashboard generation
///
/// **Fixed Time**: Oct 25, 2025, 1:00 AM UTC = Oct 25, 2025, 12:00 PM Melbourne (AEDT)
///
/// **Tests**: API parsing, float temps, lat/lon provider, timezone conversion, dashboard rendering
#[tokio::test]
#[serial_test::serial]
async fn snapshot_open_meteo_dashboard() {
    let svg = run_open_meteo_snapshot_test(TestCase {
        fixture_path: test_utils::fixtures::OPEN_METEO,
        clock_time: "2025-10-25T01:00:00Z",
        timezone: None,
        output_name: "dashboard",
    })
    .await;

    if !svg.is_empty() {
        insta::assert_snapshot!(svg);
    }
}

/// Test Open-Meteo at midnight boundary (date transition edge case)
///
/// **Fixed Time**: Oct 26, 2025, 00:00:00 UTC = Oct 26, 2025, 11:00 AM Melbourne (AEDT)
///
/// **Tests**: Date transitions, daily forecast alignment, hourly graph starting hour
#[tokio::test]
#[serial_test::serial]
async fn snapshot_open_meteo_midnight_boundary() {
    let svg = run_open_meteo_snapshot_test(TestCase {
        fixture_path: test_utils::fixtures::OPEN_METEO,
        clock_time: "2025-10-26T00:00:00Z",
        timezone: None,
        output_name: "midnight_boundary",
    })
    .await;

    if !svg.is_empty() {
        insta::assert_snapshot!(svg);
    }
}

/// Test Open-Meteo at end of day (late evening edge case)
///
/// **Fixed Time**: Oct 25, 2025, 13:00:00 UTC = Oct 26, 2025, 12:00 AM Melbourne (AEDT)
///
/// **Tests**: Local midnight rollover, "today" refers to new date
#[tokio::test]
#[serial_test::serial]
async fn snapshot_open_meteo_end_of_day() {
    let svg = run_open_meteo_snapshot_test(TestCase {
        fixture_path: test_utils::fixtures::OPEN_METEO,
        clock_time: "2025-10-25T13:00:00Z",
        timezone: None,
        output_name: "end_of_day",
    })
    .await;

    if !svg.is_empty() {
        insta::assert_snapshot!(svg);
    }
}

/// Test Open-Meteo during early morning hours
///
/// **Fixed Time**: Oct 25, 2025, 16:00:00 UTC = Oct 26, 2025, 3:00 AM Melbourne (AEDT)
///
/// **Tests**: Timezone offset (UTC+11), hour labels start from 3 AM
#[tokio::test]
#[serial_test::serial]
async fn snapshot_open_meteo_early_morning() {
    let svg = run_open_meteo_snapshot_test(TestCase {
        fixture_path: test_utils::fixtures::OPEN_METEO,
        clock_time: "2025-10-25T16:00:00Z",
        timezone: None,
        output_name: "early_morning",
    })
    .await;

    if !svg.is_empty() {
        insta::assert_snapshot!(svg);
    }
}

/// Test Open-Meteo GMT boundary in NY timezone (6pm local, before midnight UTC)
///
/// **Fixed Time**: Dec 28, 2025, 23:00:00 UTC = Dec 28, 2025, 6:00 PM NY (EST, UTC-5)
///
/// **Tests**: 1 hour before UTC midnight, daily forecast aligned with local NY calendar
#[tokio::test]
#[serial_test::serial]
async fn snapshot_open_meteo_ny_6pm_before_gmt_boundary() {
    let svg = run_open_meteo_snapshot_test(TestCase {
        fixture_path: test_utils::fixtures::NY_6PM,
        clock_time: "2025-12-28T23:00:00Z",
        timezone: Some("America/New_York"),
        output_name: "ny_6pm_before_gmt_boundary",
    })
    .await;

    if !svg.is_empty() {
        insta::assert_snapshot!(svg);
    }
}

/// Test Open-Meteo GMT boundary in NY timezone (7pm local, after midnight UTC)
///
/// **Fixed Time**: Dec 29, 2025, 00:00:00 UTC = Dec 28, 2025, 7:00 PM NY (EST, UTC-5)
///
/// **Tests**: UTC rolled to Dec 29, but still Dec 28 in NY - daily forecast consistency
#[tokio::test]
#[serial_test::serial]
async fn snapshot_open_meteo_ny_7pm_after_gmt_boundary() {
    let svg = run_open_meteo_snapshot_test(TestCase {
        fixture_path: test_utils::fixtures::NY_7PM,
        clock_time: "2025-12-29T00:00:00Z",
        timezone: Some("America/New_York"),
        output_name: "ny_7pm_after_gmt_boundary",
    })
    .await;

    if !svg.is_empty() {
        insta::assert_snapshot!(svg);
    }
}
