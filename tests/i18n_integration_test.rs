mod helpers;

use helpers::test_utils;
use helpers::wiremock_setup;
use pi_inky_weather_epd::configs::settings::HourFormat;
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
    // Chart x-axis: French defaults to 24-hour labels, not English am/pm.
    assert!(svg_content.contains("16:00</text>"));
    assert!(!svg_content.contains("pm</text>"));
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
    // Chart x-axis: German defaults to 24-hour labels, not English am/pm.
    assert!(svg_content.contains("16:00</text>"));
    assert!(!svg_content.contains("pm</text>"));
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
    // Chart x-axis: Spanish defaults to 24-hour labels, not English am/pm.
    assert!(svg_content.contains("16:00</text>"));
    assert!(!svg_content.contains("pm</text>"));
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
    assert!(svg_content.contains("指標"));
    assert!(svg_content.contains("今"));
    // Date header: Saturday 25 October in Japanese (%A, %d %B)
    assert!(svg_content.contains("土曜日, 25 10月"));
    // Tomorrow chart marker: 2025-10-26 is Sunday = 日曜日. Kanji rotated 90°
    // (the Latin-script convention other locales use here) reads as broken,
    // so Japanese renders the long-form name stacked top-to-bottom (tategaki)
    // instead — one <tspan> per character — see draw_tomorrow_line in chart.rs.
    assert!(svg_content.contains("<tspan x=\"") && svg_content.contains(">日</tspan>"));
    assert!(svg_content.contains(">曜</tspan>"));
    // Chart x-axis: Japanese defaults to 24-hour labels, not English am/pm.
    assert!(svg_content.contains("16:00</text>"));
    assert!(!svg_content.contains("pm</text>"));
}

#[tokio::test]
async fn hour_format_override_forces_twelve_hour_regardless_of_language() {
    let mock_server = wiremock_setup::setup_open_meteo_mock(
        "tests/fixtures/open_meteo_hourly_forecast.json",
        "tests/fixtures/open_meteo_daily_forecast.json",
    )
    .await;

    let mut settings = test_utils::open_meteo_settings(&mock_server.uri());
    settings.render_options.language = "fr".to_string();
    settings.render_options.hour_format = HourFormat::TwelveHour;

    let clock =
        FixedClock::from_rfc3339("2025-10-25T01:00:00Z").expect("Failed to create fixed clock");
    let output_svg_name = Path::new("tests/output/hour_format_override_twelve_hour.svg");

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

    // Forced 12-hour format overrides French's 24-hour default.
    assert!(svg_content.contains("4pm</text>"));
    assert!(!svg_content.contains("16:00</text>"));
}

#[tokio::test]
async fn hour_format_override_forces_twenty_four_hour_regardless_of_language() {
    let mock_server = wiremock_setup::setup_open_meteo_mock(
        "tests/fixtures/open_meteo_hourly_forecast.json",
        "tests/fixtures/open_meteo_daily_forecast.json",
    )
    .await;

    let mut settings = test_utils::open_meteo_settings(&mock_server.uri());
    settings.render_options.language = "en".to_string();
    settings.render_options.hour_format = HourFormat::TwentyFour;

    let clock =
        FixedClock::from_rfc3339("2025-10-25T01:00:00Z").expect("Failed to create fixed clock");
    let output_svg_name = Path::new("tests/output/hour_format_override_twenty_four_hour.svg");

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

    // Forced 24-hour format overrides English's 12-hour default.
    assert!(svg_content.contains("16:00</text>"));
    assert!(!svg_content.contains("4pm</text>"));
}
