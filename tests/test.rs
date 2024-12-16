use pi_inky_weather_epd::generate_weather_dashboard;

#[test]
fn test_weather_dashboard_ok() {
    let result = generate_weather_dashboard();
    assert!(result.is_ok());
}
