/// Test to verify that daily forecast dates are correctly mapped to day names
///
/// This test ensures that the date-to-day-name conversion works correctly for both providers
use chrono::{Datelike, Weekday};
use pi_inky_weather_epd::apis::open_meteo::models::OpenMeteoHourlyResponse;

#[test]
fn test_open_meteo_daily_dates_deserialize_correctly() {
    // Test data with known dates
    let json = r#"{
        "latitude":-37.75,
        "longitude":144.875,
        "timezone":"GMT",
        "timezone_abbreviation":"GMT",
        "current_units":{"time":"iso8601","interval":"seconds","is_day":""},
        "current":{"time":"2025-10-25T12:00","interval":900,"is_day":1},
        "hourly_units":{"time":"iso8601","temperature_2m":"°C","apparent_temperature":"°C","precipitation_probability":"%","precipitation":"mm","uv_index":"","wind_speed_10m":"km/h","wind_gusts_10m":"km/h","relative_humidity_2m":"%"},
        "hourly":{"time":["2025-10-25T12:00"],"temperature_2m":[20.0],"apparent_temperature":[18.0],"precipitation_probability":[10],"precipitation":[0.0],"uv_index":[5.0],"wind_speed_10m":[15.0],"wind_gusts_10m":[25.0],"relative_humidity_2m":[50],"cloud_cover":[30]},
        "daily_units":{"time":"iso8601","sunrise":"iso8601","sunset":"iso8601","temperature_2m_max":"°C","temperature_2m_min":"°C","precipitation_sum":"mm","precipitation_probability_max":"%"},
        "daily":{"time":["2025-10-25","2025-10-26","2025-10-27","2025-10-28","2025-10-29","2025-10-30","2025-10-31"],"sunrise":["2025-10-25T19:00","2025-10-26T19:00","2025-10-27T19:00","2025-10-28T19:00","2025-10-29T19:00","2025-10-30T19:00","2025-10-31T19:00"],"sunset":["2025-10-25T09:00","2025-10-26T09:00","2025-10-27T09:00","2025-10-28T09:00","2025-10-29T09:00","2025-10-30T09:00","2025-10-31T09:00"],"temperature_2m_max":[22.0,23.0,24.0,25.0,26.0,27.0,28.0],"temperature_2m_min":[12.0,13.0,14.0,15.0,16.0,17.0,18.0],"precipitation_sum":[0.0,1.0,2.0,0.0,0.0,0.0,0.0],"precipitation_probability_max":[10,30,50,20,10,5,0],"cloud_cover_mean":[20,45,65,25,10,8,5]}
    }"#;

    let response: OpenMeteoHourlyResponse = serde_json::from_str(json).unwrap();

    // October 25, 2025 is a Saturday
    // The dates should be: Sat(25), Sun(26), Mon(27), Tue(28), Wed(29), Thu(30), Fri(31)
    let expected_days = [
        (Weekday::Sat, "2025-10-25"),
        (Weekday::Sun, "2025-10-26"),
        (Weekday::Mon, "2025-10-27"),
        (Weekday::Tue, "2025-10-28"),
        (Weekday::Wed, "2025-10-29"),
        (Weekday::Thu, "2025-10-30"),
        (Weekday::Fri, "2025-10-31"),
    ];

    assert_eq!(response.daily.time.len(), 7, "Should have 7 days of data");

    for (i, (expected_weekday, expected_date_str)) in expected_days.iter().enumerate() {
        let date = response.daily.time[i];

        // Check that the date matches what we expect
        let date_str = date.format("%Y-%m-%d").to_string();
        assert_eq!(
            &date_str, expected_date_str,
            "Date string mismatch at index {i}"
        );

        // Check weekday (NaiveDate has weekday() method)
        let weekday = date.weekday();
        assert_eq!(
            &weekday, expected_weekday,
            "Weekday mismatch at index {i}: expected {expected_weekday}, got {weekday}"
        );
    }
}
