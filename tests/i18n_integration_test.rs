mod helpers;

use helpers::wiremock_setup;
use pi_inky_weather_epd::{clock::FixedClock, generate_weather_dashboard_injection, CONFIG};
use std::fs;
use std::path::Path;

#[tokio::test]
#[serial_test::serial]
async fn french_language_override_localizes_rendered_dashboard() {
    std::env::set_var("APP_RENDER_OPTIONS__LANGUAGE", "fr");

    let mock_server = wiremock_setup::setup_open_meteo_mock(
        "tests/fixtures/open_meteo_hourly_forecast.json",
        "tests/fixtures/open_meteo_daily_forecast.json",
    )
    .await;
    std::env::set_var("OPEN_METEO_BASE_URL", mock_server.uri());

    let clock =
        FixedClock::from_rfc3339("2025-10-25T01:00:00Z").expect("Failed to create fixed clock");
    let output_svg_name = Path::new("tests/output/french_language_dashboard.svg");

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

        fs::read_to_string(output_svg_name).expect("Failed to read generated SVG file")
    })
    .await
    .expect("Task panicked");

    std::env::remove_var("OPEN_METEO_BASE_URL");
    std::env::remove_var("APP_RENDER_OPTIONS__LANGUAGE");

    assert!(svg_content.contains("Mesure"));
    assert!(svg_content.contains("Maint."));
    assert!(svg_content.contains("Samedi, 25 Octobre"));
    assert!(svg_content.contains("Dimanche"));
    assert!(svg_content.contains("Dim"));
}

// NOTE:
// CONFIG is a global Lazy singleton initialized once per test binary. To keep this
// integration test deterministic, this file only contains one locale override test.
