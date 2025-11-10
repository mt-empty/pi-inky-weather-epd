use pi_inky_weather_epd::configs::settings::WindSpeedUnit;
use pi_inky_weather_epd::domain::models::Wind;

#[test]
fn test_wind_speed_kmh_no_conversion() {
    let speed_kmh = 20;
    let result = Wind::convert_speed(speed_kmh, WindSpeedUnit::KmH);
    assert_eq!(result, 20);
}

#[test]
fn test_wind_speed_kmh_to_mph() {
    let speed_kmh = 20;
    let result = Wind::convert_speed(speed_kmh, WindSpeedUnit::Mph);
    // 20 km/h * 0.621371 = 12.42742 ≈ 12 mph
    assert_eq!(result, 12);
}

#[test]
fn test_wind_speed_kmh_to_knots() {
    let speed_kmh = 20;
    let result = Wind::convert_speed(speed_kmh, WindSpeedUnit::Knots);
    // 20 km/h * 0.539957 = 10.79914 ≈ 11 knots
    assert_eq!(result, 11);
}

#[test]
fn test_wind_speed_zero() {
    assert_eq!(Wind::convert_speed(0, WindSpeedUnit::KmH), 0);
    assert_eq!(Wind::convert_speed(0, WindSpeedUnit::Mph), 0);
    assert_eq!(Wind::convert_speed(0, WindSpeedUnit::Knots), 0);
}

#[test]
fn test_wind_speed_high_values() {
    // Test with 100 km/h
    let speed_kmh = 100;

    let kmh = Wind::convert_speed(speed_kmh, WindSpeedUnit::KmH);
    let mph = Wind::convert_speed(speed_kmh, WindSpeedUnit::Mph);
    let knots = Wind::convert_speed(speed_kmh, WindSpeedUnit::Knots);

    assert_eq!(kmh, 100);
    assert_eq!(mph, 62); // 100 * 0.621371 = 62.1371 ≈ 62
    assert_eq!(knots, 54); // 100 * 0.539957 = 53.9957 ≈ 54
}

#[test]
fn test_get_speed_in_unit_without_gust() {
    let wind = Wind::new(20, 30);

    assert_eq!(wind.get_speed_in_unit(false, WindSpeedUnit::KmH), 20);
    assert_eq!(wind.get_speed_in_unit(false, WindSpeedUnit::Mph), 12);
    assert_eq!(wind.get_speed_in_unit(false, WindSpeedUnit::Knots), 11);
}

#[test]
fn test_get_speed_in_unit_with_gust() {
    let wind = Wind::new(20, 30);

    assert_eq!(wind.get_speed_in_unit(true, WindSpeedUnit::KmH), 30);
    assert_eq!(wind.get_speed_in_unit(true, WindSpeedUnit::Mph), 19); // 30 * 0.621371 = 18.64113 ≈ 19
    assert_eq!(wind.get_speed_in_unit(true, WindSpeedUnit::Knots), 16); // 30 * 0.539957 = 16.19871 ≈ 16
}

#[test]
fn test_conversion_factors_accuracy() {
    // Verify conversion factors are accurate
    // 1 km/h = 0.621371 mph
    // 1 km/h = 0.539957 knots

    let test_cases = vec![
        (10, 6, 5),    // 10 km/h ≈ 6 mph ≈ 5 knots
        (50, 31, 27),  // 50 km/h ≈ 31 mph ≈ 27 knots
        (80, 50, 43),  // 80 km/h ≈ 50 mph ≈ 43 knots
        (120, 75, 65), // 120 km/h ≈ 75 mph ≈ 65 knots
    ];

    for (kmh, expected_mph, expected_knots) in test_cases {
        assert_eq!(
            Wind::convert_speed(kmh, WindSpeedUnit::Mph),
            expected_mph,
            "Failed for {kmh} km/h to mph"
        );
        assert_eq!(
            Wind::convert_speed(kmh, WindSpeedUnit::Knots),
            expected_knots,
            "Failed for {kmh} km/h to knots"
        );
    }
}
