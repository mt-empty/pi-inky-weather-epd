mod helpers;

use helpers::test_utils;
use helpers::wiremock_setup;
use pi_inky_weather_epd::{clock::FixedClock, generate_weather_dashboard_injection};
use std::fs;
use std::path::Path;

#[tokio::test]
async fn french_language_override_localizes_rendered_dashboard() {
    let mock_server = wiremock_setup::setup_open_meteo_mock(
        "tests/fixtures/open_meteo_hourly_forecast.json",
        "tests/fixtures/open_meteo_daily_forecast.json",
    )
    .await;

    let mut settings = test_utils::open_meteo_settings(&mock_server.uri());
    settings.render_options.language = "fr".to_string();

    let clock =
        FixedClock::from_rfc3339("2025-10-25T01:00:00Z").expect("Failed to create fixed clock");
    let output_svg_name = Path::new("tests/output/french_language_dashboard.svg");

    let svg_content = tokio::task::spawn_blocking(move || {
        let result = generate_weather_dashboard_injection(&settings, &clock, output_svg_name);
        assert!(
            result.is_ok(),
            "Dashboard generation failed: {:?}",
            result.err()
        );

        fs::read_to_string(output_svg_name).expect("Failed to read generated SVG file")
    })
    .await
    .expect("Task panicked");

    assert!(svg_content.contains("Mesure"));
    assert!(svg_content.contains("Maint."));
    assert!(svg_content.contains("Samedi, 25 Octobre"));
    assert!(svg_content.contains("Dimanche"));
    assert!(svg_content.contains("Dim"));
}

#[tokio::test]
async fn german_language_override_localizes_rendered_dashboard() {
    let mock_server = wiremock_setup::setup_open_meteo_mock(
        "tests/fixtures/open_meteo_hourly_forecast.json",
        "tests/fixtures/open_meteo_daily_forecast.json",
    )
    .await;

    let mut settings = test_utils::open_meteo_settings(&mock_server.uri());
    settings.render_options.language = "de".to_string();

    let clock =
        FixedClock::from_rfc3339("2025-10-25T01:00:00Z").expect("Failed to create fixed clock");
    let output_svg_name = Path::new("tests/output/german_language_dashboard.svg");

    let svg_content = tokio::task::spawn_blocking(move || {
        let result = generate_weather_dashboard_injection(&settings, &clock, output_svg_name);
        assert!(
            result.is_ok(),
            "Dashboard generation failed: {:?}",
            result.err()
        );

        fs::read_to_string(output_svg_name).expect("Failed to read generated SVG file")
    })
    .await
    .expect("Task panicked");

    // Labels
    assert!(svg_content.contains("Wert"));
    assert!(svg_content.contains("Jetzt"));
    // Date header: Saturday 25 October in German (%A, %d %B)
    assert!(svg_content.contains("Samstag, 25 Oktober"));
    // Tomorrow chart marker: 2025-10-26 is Sunday = Sonntag
    assert!(svg_content.contains("Sonntag"));
}

#[tokio::test]
async fn spanish_language_override_localizes_rendered_dashboard() {
    let mock_server = wiremock_setup::setup_open_meteo_mock(
        "tests/fixtures/open_meteo_hourly_forecast.json",
        "tests/fixtures/open_meteo_daily_forecast.json",
    )
    .await;

    let mut settings = test_utils::open_meteo_settings(&mock_server.uri());
    settings.render_options.language = "es".to_string();

    let clock =
        FixedClock::from_rfc3339("2025-10-25T01:00:00Z").expect("Failed to create fixed clock");
    let output_svg_name = Path::new("tests/output/spanish_language_dashboard.svg");

    let svg_content = tokio::task::spawn_blocking(move || {
        let result = generate_weather_dashboard_injection(&settings, &clock, output_svg_name);
        assert!(
            result.is_ok(),
            "Dashboard generation failed: {:?}",
            result.err()
        );

        fs::read_to_string(output_svg_name).expect("Failed to read generated SVG file")
    })
    .await
    .expect("Task panicked");

    // Labels
    assert!(svg_content.contains("Medida"));
    assert!(svg_content.contains("Ahora"));
    // Date header: Saturday 25 October in Spanish (%A, %d %B)
    assert!(svg_content.contains("Sábado, 25 Octubre"));
    // Tomorrow chart marker: 2025-10-26 is Sunday = Domingo
    assert!(svg_content.contains("Domingo"));
}

#[tokio::test]
async fn japanese_language_override_localizes_rendered_dashboard() {
    let mock_server = wiremock_setup::setup_open_meteo_mock(
        "tests/fixtures/open_meteo_hourly_forecast.json",
        "tests/fixtures/open_meteo_daily_forecast.json",
    )
    .await;

    let mut settings = test_utils::open_meteo_settings(&mock_server.uri());
    settings.render_options.language = "ja".to_string();

    let clock =
        FixedClock::from_rfc3339("2025-10-25T01:00:00Z").expect("Failed to create fixed clock");
    let output_svg_name = Path::new("tests/output/japanese_language_dashboard.svg");

    let svg_content = tokio::task::spawn_blocking(move || {
        let result = generate_weather_dashboard_injection(&settings, &clock, output_svg_name);
        assert!(
            result.is_ok(),
            "Dashboard generation failed: {:?}",
            result.err()
        );

        fs::read_to_string(output_svg_name).expect("Failed to read generated SVG file")
    })
    .await
    .expect("Task panicked");

    // Labels
    assert!(svg_content.contains("Shihyo"));
    assert!(svg_content.contains("Ima"));
    // Date header: Saturday 25 October in Japanese (%A, %d %B)
    assert!(svg_content.contains("Doyobi, 25 Jugatsu"));
    // Tomorrow chart marker: 2025-10-26 is Sunday = Nichiyobi
    assert!(svg_content.contains("Nichiyobi"));
}
