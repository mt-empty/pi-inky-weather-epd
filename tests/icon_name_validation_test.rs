use chrono::Utc;
/// Test to verify that icon names generated for forecasts are valid
/// and correspond to files that actually exist in the static directory.
///
/// This test was created to prevent the bug where low precipitation chance
/// (0-25% = "Clear") combined with some precipitation amount would generate
/// invalid icon names like "clear-day-drizzle.svg" which don't exist.
use pi_inky_weather_epd::domain::models::{
    DailyForecast, HourlyForecast, Precipitation, Temperature, Wind,
};
use pi_inky_weather_epd::weather::icons::Icon;

#[test]
fn test_clear_sky_never_has_precipitation_suffix_hourly() {
    // Test case: Low chance (0-25%) with some amount should produce "clear-day.svg"
    // not "clear-day-drizzle.svg"

    let forecast = HourlyForecast {
        time: Utc::now(),
        temperature: Temperature::celsius(20.0),
        apparent_temperature: Temperature::celsius(18.0),
        wind: Wind::new(10, 15),
        precipitation: Precipitation::new(
            Some(20), // 20% chance (Clear range: 0-25%)
            Some(0),
            Some(10), // 10mm which would be 240mm/day -> "Rain" if not cleared
        ),
        uv_index: 5,
        relative_humidity: 50,
        is_night: false,
    };

    let icon_name = forecast.get_icon_name();

    // Should be "clear-day.svg", not "clear-day-rain.svg"
    assert_eq!(icon_name, "clear-day.svg");
}

#[test]
fn test_clear_sky_never_has_precipitation_suffix_hourly_night() {
    // Test case: Low chance at night with drizzle amount
    // Note: This test may return a moon phase icon if use_moon_phase_instead_of_clear_night is enabled
    // The important thing is it should NOT be "clear-night-drizzle.svg"

    let forecast = HourlyForecast {
        time: Utc::now(),
        temperature: Temperature::celsius(15.0),
        apparent_temperature: Temperature::celsius(13.0),
        wind: Wind::new(5, 8),
        precipitation: Precipitation::new(
            Some(10), // 10% chance (Clear range: 0-25%)
            Some(0),
            Some(1), // 1mm which would be 24mm/day -> "Drizzle" if not cleared
        ),
        uv_index: 0,
        relative_humidity: 60,
        is_night: true,
    };

    let icon_name = forecast.get_icon_name();

    // Should be "clear-night.svg" or a moon phase icon, but NOT "clear-night-drizzle.svg"
    assert!(
        icon_name == "clear-night.svg" || icon_name.starts_with("moon-"),
        "Expected clear-night.svg or moon-*.svg, got: {icon_name}"
    );
}

#[test]
fn test_clear_sky_never_has_precipitation_suffix_daily() {
    // Test case: Low chance for daily forecast with some amount

    let forecast = DailyForecast {
        date: Some(Utc::now().date_naive()),
        temp_max: Some(Temperature::celsius(25.0)),
        temp_min: Some(Temperature::celsius(15.0)),
        precipitation: Some(Precipitation::new(
            Some(15), // 15% chance (Clear range: 0-25%)
            Some(0),
            Some(5), // 5mm which would be "Drizzle" if not cleared
        )),
        astronomical: None,
    };

    let icon_name = forecast.get_icon_name();

    // Should be "clear-day.svg", not "clear-day-drizzle.svg"
    assert_eq!(icon_name, "clear-day.svg");
}

#[test]
fn test_partly_cloudy_with_drizzle_is_valid() {
    // Test case: Partly cloudy (26-50%) with drizzle should work fine

    let forecast = HourlyForecast {
        time: Utc::now(),
        temperature: Temperature::celsius(18.0),
        apparent_temperature: Temperature::celsius(16.0),
        wind: Wind::new(12, 20),
        precipitation: Precipitation::new(
            Some(40), // 40% chance (PartlyCloudy range: 26-50%)
            Some(0),
            Some(1), // 1mm which would be 24mm/day -> "Drizzle"
        ),
        uv_index: 3,
        relative_humidity: 70,
        is_night: false,
    };

    let icon_name = forecast.get_icon_name();

    // Should be "partly-cloudy-day-drizzle.svg" - this file exists
    assert_eq!(icon_name, "partly-cloudy-day-drizzle.svg");
}

#[test]
fn test_overcast_with_rain_is_valid() {
    // Test case: Overcast (51-75%) with rain should work fine

    let forecast = HourlyForecast {
        time: Utc::now(),
        temperature: Temperature::celsius(16.0),
        apparent_temperature: Temperature::celsius(14.0),
        wind: Wind::new(15, 25),
        precipitation: Precipitation::new(
            Some(60), // 60% chance (Overcast range: 51-75%)
            Some(5),
            Some(15), // 10mm median which would be 240mm/day -> "Rain"
        ),
        uv_index: 2,
        relative_humidity: 85,
        is_night: false,
    };

    let icon_name = forecast.get_icon_name();

    // Should be "overcast-day-rain.svg" - this file exists
    assert_eq!(icon_name, "overcast-day-rain.svg");
}

#[test]
fn test_extreme_with_drizzle_is_valid() {
    // Test case: Extreme chance (76%+) with drizzle amount

    let forecast = HourlyForecast {
        time: Utc::now(),
        temperature: Temperature::celsius(12.0),
        apparent_temperature: Temperature::celsius(10.0),
        wind: Wind::new(20, 35),
        precipitation: Precipitation::new(
            Some(85), // 85% chance (Extreme range: 76%+)
            Some(0),
            Some(1), // 0.5mm median which would be 12mm/day -> "Drizzle" (3-20mm range)
        ),
        uv_index: 1,
        relative_humidity: 90,
        is_night: true,
    };

    let icon_name = forecast.get_icon_name();

    // Should be "extreme-night-drizzle.svg" - this file exists
    assert_eq!(icon_name, "extreme-night-drizzle.svg");
}

#[test]
fn test_zero_chance_zero_amount_produces_clear() {
    // Edge case: No precipitation at all

    let forecast = DailyForecast {
        date: Some(Utc::now().date_naive()),
        temp_max: Some(Temperature::celsius(28.0)),
        temp_min: Some(Temperature::celsius(18.0)),
        precipitation: Some(Precipitation::new(
            Some(0), // 0% chance
            Some(0),
            Some(0), // 0mm
        )),
        astronomical: None,
    };

    let icon_name = forecast.get_icon_name();

    assert_eq!(icon_name, "clear-day.svg");
}

#[test]
fn test_boundary_case_25_percent_is_still_clear() {
    // Boundary test: 25% is the upper limit of Clear (0-25%)

    let forecast = HourlyForecast {
        time: Utc::now(),
        temperature: Temperature::celsius(20.0),
        apparent_temperature: Temperature::celsius(19.0),
        wind: Wind::new(8, 12),
        precipitation: Precipitation::new(
            Some(25), // 25% chance - still in Clear range
            Some(0),
            Some(5), // Would be "Drizzle" if amount mattered
        ),
        uv_index: 4,
        relative_humidity: 55,
        is_night: false,
    };

    let icon_name = forecast.get_icon_name();

    // 25% is still Clear, so should ignore the amount
    assert_eq!(icon_name, "clear-day.svg");
}

#[test]
fn test_boundary_case_26_percent_allows_precipitation_suffix() {
    // Boundary test: 26% is PartlyCloudy (26-50%) so drizzle suffix is allowed

    let forecast = HourlyForecast {
        time: Utc::now(),
        temperature: Temperature::celsius(20.0),
        apparent_temperature: Temperature::celsius(19.0),
        wind: Wind::new(8, 12),
        precipitation: Precipitation::new(
            Some(26), // 26% chance - PartlyCloudy range
            Some(0),
            Some(1), // Would be "Drizzle" (24mm/day)
        ),
        uv_index: 4,
        relative_humidity: 55,
        is_night: false,
    };

    let icon_name = forecast.get_icon_name();

    // 26% is PartlyCloudy, so drizzle suffix should appear
    assert_eq!(icon_name, "partly-cloudy-day-drizzle.svg");
}
