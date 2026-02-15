use chrono::Utc;
/// Tests for icon name generation with cloud cover data and precipitation override logic.
///
/// The icon selection system now prioritizes cloud_cover data when available,
/// with fallback to precipitation-based estimation. The precipitation override
/// ensures realistic weather combinations (e.g., no heavy rain with clear skies).
use pi_inky_weather_epd::domain::models::{
    DailyForecast, HourlyForecast, Precipitation, Temperature, Wind,
};
use pi_inky_weather_epd::weather::icons::Icon;

// ============================================================================
// Cloud Cover Feature Tests
// ============================================================================

#[test]
fn test_cloud_cover_overrides_low_precipitation_chance() {
    // High cloud cover (80%) should produce overcast icon even with low precip chance (10%)
    let forecast = HourlyForecast {
        time: Utc::now(),
        temperature: Temperature::celsius(18.0),
        apparent_temperature: Temperature::celsius(16.0),
        wind: Wind::new(10, 15),
        precipitation: Precipitation::new(
            Some(10), // Low chance - would be Clear if using fallback
            Some(0),
            Some(0),
        ),
        uv_index: 3,
        relative_humidity: 70,
        is_night: false,
        cloud_cover: Some(80), // High cloud cover - should override
        weather_code: None,
    };

    assert_eq!(forecast.get_icon_name(), "extreme-day.svg");
}

#[test]
fn test_cloud_cover_boundary_25_percent() {
    // 25% cloud cover is upper limit of Clear range
    let forecast_25 = HourlyForecast {
        time: Utc::now(),
        temperature: Temperature::celsius(20.0),
        apparent_temperature: Temperature::celsius(19.0),
        wind: Wind::new(8, 12),
        precipitation: Precipitation::new(Some(50), Some(0), Some(0)),
        uv_index: 5,
        relative_humidity: 50,
        is_night: false,
        cloud_cover: Some(25), // Boundary - still Clear
        weather_code: None,
    };

    let forecast_26 = HourlyForecast {
        cloud_cover: Some(26), // Just over boundary - now PartlyCloudy
        ..forecast_25.clone()
    };

    assert_eq!(forecast_25.get_icon_name(), "clear-day.svg");
    assert_eq!(forecast_26.get_icon_name(), "partly-cloudy-day.svg");
}

#[test]
fn test_cloud_cover_boundary_50_and_51_percent() {
    // Test boundary between PartlyCloudy (50%) and Overcast (51%)
    let forecast_50 = HourlyForecast {
        time: Utc::now(),
        temperature: Temperature::celsius(18.0),
        apparent_temperature: Temperature::celsius(17.0),
        wind: Wind::new(10, 15),
        precipitation: Precipitation::new(Some(30), Some(0), Some(0)),
        uv_index: 3,
        relative_humidity: 65,
        is_night: false,
        cloud_cover: Some(50),
        weather_code: None,
    };

    let forecast_51 = HourlyForecast {
        cloud_cover: Some(51),
        ..forecast_50.clone()
    };

    assert_eq!(forecast_50.get_icon_name(), "partly-cloudy-day.svg");
    assert_eq!(forecast_51.get_icon_name(), "overcast-day.svg");
}

#[test]
fn test_cloud_cover_boundary_75_and_76_percent() {
    // Test boundary between Overcast (75%) and Extreme (76%)
    let forecast_75 = HourlyForecast {
        time: Utc::now(),
        temperature: Temperature::celsius(16.0),
        apparent_temperature: Temperature::celsius(15.0),
        wind: Wind::new(12, 20),
        precipitation: Precipitation::new(Some(60), Some(0), Some(0)),
        uv_index: 2,
        relative_humidity: 80,
        is_night: false,
        cloud_cover: Some(75),
        weather_code: None,
    };

    let forecast_76 = HourlyForecast {
        cloud_cover: Some(76),
        ..forecast_75.clone()
    };

    assert_eq!(forecast_75.get_icon_name(), "overcast-day.svg");
    assert_eq!(forecast_76.get_icon_name(), "extreme-day.svg");
}

#[test]
fn test_null_cloud_cover_falls_back_to_precipitation() {
    // When cloud_cover is None, should use precipitation chance for cloud level
    let forecast = HourlyForecast {
        time: Utc::now(),
        temperature: Temperature::celsius(20.0),
        apparent_temperature: Temperature::celsius(19.0),
        wind: Wind::new(10, 15),
        precipitation: Precipitation::new(
            Some(40), // PartlyCloudy range
            Some(0),
            Some(0),
        ),
        uv_index: 4,
        relative_humidity: 60,
        is_night: false,
        cloud_cover: None, // Fallback to precipitation
        weather_code: None,
    };

    assert_eq!(forecast.get_icon_name(), "partly-cloudy-day.svg");
}

// ============================================================================
// Precipitation Override Tests
// ============================================================================

#[test]
fn test_precipitation_override_drizzle_requires_partly_cloudy() {
    // Clear skies (20% cloud) + drizzle should be bumped to PartlyCloudy
    let forecast = HourlyForecast {
        time: Utc::now(),
        temperature: Temperature::celsius(18.0),
        apparent_temperature: Temperature::celsius(17.0),
        wind: Wind::new(8, 12),
        precipitation: Precipitation::new(
            Some(20),
            Some(0),
            Some(1), // 24mm/day = Drizzle
        ),
        uv_index: 4,
        relative_humidity: 65,
        is_night: false,
        cloud_cover: Some(15), // Clear range, but drizzle present
        weather_code: None,
    };

    // Should be bumped to partly-cloudy due to drizzle
    assert_eq!(forecast.get_icon_name(), "partly-cloudy-day-drizzle.svg");
}

#[test]
fn test_precipitation_override_rain_requires_overcast() {
    // Clear skies (20% cloud) + heavy rain should be bumped to Overcast
    let forecast = HourlyForecast {
        time: Utc::now(),
        temperature: Temperature::celsius(16.0),
        apparent_temperature: Temperature::celsius(15.0),
        wind: Wind::new(15, 25),
        precipitation: Precipitation::new(
            Some(25),
            Some(10),
            Some(20), // 360mm/day = Rain
        ),
        uv_index: 2,
        relative_humidity: 85,
        is_night: false,
        cloud_cover: Some(20), // Clear range, but heavy rain present
        weather_code: None,
    };

    // Should be bumped to overcast due to heavy rain
    assert_eq!(forecast.get_icon_name(), "overcast-day-rain.svg");
}

#[test]
fn test_precipitation_override_partly_cloudy_rain_becomes_overcast() {
    // PartlyCloudy (40% cloud) + heavy rain should be bumped to Overcast
    let forecast = HourlyForecast {
        time: Utc::now(),
        temperature: Temperature::celsius(16.0),
        apparent_temperature: Temperature::celsius(14.0),
        wind: Wind::new(18, 30),
        precipitation: Precipitation::new(
            Some(50),
            Some(8),
            Some(15), // 276mm/day = Rain
        ),
        uv_index: 1,
        relative_humidity: 90,
        is_night: false,
        cloud_cover: Some(40), // PartlyCloudy range, but heavy rain present
        weather_code: None,
    };

    // Should be bumped to overcast due to heavy rain
    assert_eq!(forecast.get_icon_name(), "overcast-day-rain.svg");
}

#[test]
fn test_fallback_with_precipitation_override() {
    // Test fallback behaviour: low precipitation chance + drizzle amount
    // Median of 0-5mm = 2.5mm which is in None range (0-2.0), so no override needed
    let forecast = DailyForecast {
        date: Some(chrono::Local::now().date_naive()),
        temp_max: Some(Temperature::celsius(25.0)),
        temp_min: Some(Temperature::celsius(15.0)),
        precipitation: Some(Precipitation::new(
            Some(15), // Clear range
            Some(0),
            Some(5), // Median 2.5mm = None range
        )),
        astronomical: None,
        cloud_cover: None, // Fallback to precipitation
        weather_code: None,
    };

    assert_eq!(forecast.get_icon_name(), "clear-day.svg");
}

// ============================================================================
// Valid Combination Tests (Existing)
// ============================================================================

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
        cloud_cover: None,
        weather_code: None,
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
        cloud_cover: None,
        weather_code: None,
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
        cloud_cover: None,
        weather_code: None,
    };

    let icon_name = forecast.get_icon_name();

    // Should be "extreme-night-drizzle.svg" - this file exists
    assert_eq!(icon_name, "extreme-night-drizzle.svg");
}

#[test]
fn test_zero_chance_zero_amount_produces_clear() {
    // Edge case: No precipitation at all

    let forecast = DailyForecast {
        date: Some(chrono::Local::now().date_naive()),
        temp_max: Some(Temperature::celsius(28.0)),
        temp_min: Some(Temperature::celsius(18.0)),
        precipitation: Some(Precipitation::new(
            Some(0), // 0% chance
            Some(0),
            Some(0), // 0mm
        )),
        astronomical: None,
        cloud_cover: None,
        weather_code: None,
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
            Some(0), // 0mm - no precipitation
        ),
        uv_index: 4,
        relative_humidity: 55,
        is_night: false,
        cloud_cover: Some(22), // Explicitly set low cloud cover to test clear sky logic
        weather_code: None,
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
        cloud_cover: None,
        weather_code: None,
    };

    let icon_name = forecast.get_icon_name();

    // 26% is PartlyCloudy, so drizzle suffix should appear
    assert_eq!(icon_name, "partly-cloudy-day-drizzle.svg");
}

// ============================================================================
// Snow Icon Selection Tests
// ============================================================================

#[test]
fn test_snow_icon_selected_with_high_snowfall() {
    // Heavy snow: 14.3cm snow = ~10mm water, total ~10mm = 100% snow
    let forecast = HourlyForecast {
        time: Utc::now(),
        temperature: Temperature::celsius(-2.0),
        apparent_temperature: Temperature::celsius(-5.0),
        wind: Wind::new(10, 15),
        precipitation: Precipitation::new_with_snowfall(
            Some(80), // Extreme range
            Some(8),
            Some(12),
            Some(143), // 14.3cm snow = ~10mm water
        ),
        uv_index: 1,
        relative_humidity: 85,
        is_night: false,
        cloud_cover: Some(80),
        weather_code: None,
    };

    assert_eq!(
        forecast.get_icon_name(),
        "extreme-day-snow.svg",
        "Heavy snowfall should produce snow icon"
    );
}

#[test]
fn test_snow_icon_at_60_percent_threshold() {
    // Boundary: exactly 60% snow (8.6cm = ~6mm water, total 10mm)
    let forecast = HourlyForecast {
        time: Utc::now(),
        temperature: Temperature::celsius(0.0),
        apparent_temperature: Temperature::celsius(-2.0),
        wind: Wind::new(8, 12),
        precipitation: Precipitation::new_with_snowfall(
            Some(65),
            Some(9),
            Some(11),
            Some(86), // 8.6cm = ~6mm water, total 10mm = 60% snow
        ),
        uv_index: 2,
        relative_humidity: 80,
        is_night: true,
        cloud_cover: Some(70),
        weather_code: None,
    };

    assert_eq!(
        forecast.get_icon_name(),
        "overcast-night-snow.svg",
        "Exactly 60% snow threshold should produce snow icon"
    );
}

#[test]
fn test_rain_icon_below_snow_threshold() {
    // ~43% snow -> should show rain (3cm = ~4.3mm water, total 10mm)
    let forecast = HourlyForecast {
        time: Utc::now(),
        temperature: Temperature::celsius(2.0),
        apparent_temperature: Temperature::celsius(0.0),
        wind: Wind::new(12, 18),
        precipitation: Precipitation::new_with_snowfall(
            Some(70),
            Some(9),
            Some(11),
            Some(3), // 3cm × 1.43 = 4.29mm water, total 10mm = 42.9% snow (below 60%)
        ),
        uv_index: 1,
        relative_humidity: 85,
        is_night: false,
        cloud_cover: Some(75),
        weather_code: None,
    };

    // Below 60% threshold, should be rain not snow
    assert_eq!(
        forecast.get_icon_name(),
        "overcast-day-rain.svg",
        "Below 60% snow threshold should produce rain icon"
    );
}

#[test]
fn test_snow_override_requires_partly_cloudy() {
    // Snow with clear skies should bump to partly-cloudy
    let forecast = HourlyForecast {
        time: Utc::now(),
        temperature: Temperature::celsius(-3.0),
        apparent_temperature: Temperature::celsius(-6.0),
        wind: Wind::new(5, 8),
        precipitation: Precipitation::new_with_snowfall(
            Some(30),
            Some(0),
            Some(2),
            Some(15), // 1.5cm = ~1.05mm water, total ~1mm = 100% snow
        ),
        uv_index: 2,
        relative_humidity: 70,
        is_night: false,
        cloud_cover: Some(20), // Clear range
        weather_code: None,
    };

    // Should be bumped to partly-cloudy due to snow
    assert_eq!(
        forecast.get_icon_name(),
        "partly-cloudy-day-snow.svg",
        "Snow with clear skies should be upgraded to partly-cloudy"
    );
}

#[test]
fn test_low_snowfall_shows_clear_not_snow() {
    // Very light snowfall below 1.4mm threshold (0.2cm = ~0.14mm water)
    let forecast = HourlyForecast {
        time: Utc::now(),
        temperature: Temperature::celsius(-1.0),
        apparent_temperature: Temperature::celsius(-3.0),
        wind: Wind::new(3, 5),
        precipitation: Precipitation::new_with_snowfall(
            Some(15),
            Some(0),
            Some(0),
            Some(2), // 0.2cm = ~0.14mm water (very light)
        ),
        uv_index: 3,
        relative_humidity: 60,
        is_night: false,
        cloud_cover: Some(10),
        weather_code: None,
    };

    // Below 1.4mm threshold for snow, should show clear
    assert_eq!(
        forecast.get_icon_name(),
        "clear-day.svg",
        "Very light snow below threshold should show clear icon"
    );
}

#[test]
fn test_mixed_precipitation_favors_rain() {
    // Mixed: ~29% snow, 71% rain (2cm = ~2.9mm water, total ~10mm)
    let forecast = HourlyForecast {
        time: Utc::now(),
        temperature: Temperature::celsius(3.0),
        apparent_temperature: Temperature::celsius(1.0),
        wind: Wind::new(10, 15),
        precipitation: Precipitation::new_with_snowfall(
            Some(60),
            Some(8),
            Some(12),
            Some(2), // 2cm × 1.43 = 2.86mm water, total 10mm = 28.6% snow (below 60%)
        ),
        uv_index: 1,
        relative_humidity: 90,
        is_night: true,
        cloud_cover: Some(65),
        weather_code: None,
    };

    // Below 60% snow threshold -> rain
    assert_eq!(
        forecast.get_icon_name(),
        "overcast-night-rain.svg",
        "Mixed precipitation below 60% snow should show rain icon"
    );
}

#[test]
fn test_partly_cloudy_snow_at_night() {
    // Light snow at night (10cm = ~7mm water, total ~8mm = 87.5% snow)
    let forecast = HourlyForecast {
        time: Utc::now(),
        temperature: Temperature::celsius(-5.0),
        apparent_temperature: Temperature::celsius(-8.0),
        wind: Wind::new(6, 10),
        precipitation: Precipitation::new_with_snowfall(
            Some(45), // PartlyCloudy range
            Some(7),
            Some(9),
            Some(100), // 10cm snow
        ),
        uv_index: 0,
        relative_humidity: 80,
        is_night: true,
        cloud_cover: Some(40),
        weather_code: None,
    };

    assert_eq!(
        forecast.get_icon_name(),
        "partly-cloudy-night-snow.svg",
        "Partly cloudy with snow at night should produce night snow icon"
    );
}

#[test]
fn test_overcast_day_snow() {
    // Overcast day with snow (11.5cm = ~8mm water, total ~9mm = 89% snow)
    let forecast = HourlyForecast {
        time: Utc::now(),
        temperature: Temperature::celsius(-4.0),
        apparent_temperature: Temperature::celsius(-7.0),
        wind: Wind::new(12, 18),
        precipitation: Precipitation::new_with_snowfall(
            Some(60), // Overcast range
            Some(8),
            Some(10),
            Some(115), // 11.5cm snow
        ),
        uv_index: 1,
        relative_humidity: 85,
        is_night: false,
        cloud_cover: Some(60),
        weather_code: None,
    };

    assert_eq!(
        forecast.get_icon_name(),
        "overcast-day-snow.svg",
        "Overcast day with snow should produce overcast-day-snow icon"
    );
}

#[test]
fn test_extreme_night_snow() {
    // Extreme conditions with snow (20cm = ~14mm water, total ~15mm = 93% snow)
    let forecast = HourlyForecast {
        time: Utc::now(),
        temperature: Temperature::celsius(-10.0),
        apparent_temperature: Temperature::celsius(-15.0),
        wind: Wind::new(20, 35),
        precipitation: Precipitation::new_with_snowfall(
            Some(90), // Extreme range
            Some(14),
            Some(16),
            Some(200), // 20cm snow
        ),
        uv_index: 0,
        relative_humidity: 90,
        is_night: true,
        cloud_cover: Some(85),
        weather_code: None,
    };

    assert_eq!(
        forecast.get_icon_name(),
        "extreme-night-snow.svg",
        "Extreme night conditions with heavy snow should produce extreme-night-snow icon"
    );
}
