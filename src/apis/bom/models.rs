// #![allow(dead_code)]
use super::utils::*;
use std::{
    fmt::{self, Display},
    ops::Deref,
};

use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::configs::settings::TemperatureUnit;

#[derive(Deserialize, Debug, Copy, PartialOrd, PartialEq, Default, Clone)]
pub struct RelativeHumidity(pub u16);

#[derive(Deserialize, Debug, Copy, PartialOrd, PartialEq, Default, Clone)]
pub struct HourlyUV(pub u16);

#[derive(Deserialize, Debug, PartialOrd, PartialEq, Clone, Copy)]
pub struct Temperature {
    pub value: f32,
    pub unit: TemperatureUnit,
}

impl Deref for Temperature {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl From<Temperature> for i16 {
    fn from(t: Temperature) -> i16 {
        t.value as i16
    }
}

impl Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rounded: i16 = self.value.round() as i16;
        write!(f, "{rounded}")
    }
}

#[derive(Deserialize, Debug)]
pub struct Wind {
    pub speed_kilometre: u16,
    // pub speed_knot: u16,
    // pub direction: String,
    // pub gust_speed_knot: Option<u16>,
    pub gust_speed_kilometre: u16,
}

// #[derive(Deserialize, Debug)]
// pub struct HourlyMetadata {
//     pub response_timestamp: DateTime<Utc>,
//     pub issue_time: DateTime<Utc>,
//     pub observation_time: Option<DateTime<Utc>>,
//     pub copyright: String,
// }

#[derive(Deserialize, Debug)]
pub struct RainAmount {
    pub min: Option<u16>,
    pub max: Option<u16>,
    // pub lower_range: Option<u16>,
    // pub upper_range: Option<u16>,
    // pub units: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Rain {
    pub amount: RainAmount,
    pub chance: Option<u16>,
    // pub chance_of_no_rain_category: Option<String>,
    // pub precipitation_amount_25_percent_chance: Option<u16>,
    // pub precipitation_amount_50_percent_chance: Option<u16>,
    // pub precipitation_amount_75_percent_chance: Option<u16>,
}

// #[derive(Deserialize, Debug)]
// pub struct UV {
//     pub category: Option<String>,
//     pub end_time: Option<DateTime<Utc>>,
//     pub max_index: Option<u16>,
//     #[serde(deserialize_with = "deserialize_optional_naive_date")]
//     pub start_time: Option<DateTime<Utc>>,
// }

#[derive(Deserialize, Debug, Default, Copy, Clone)]
pub struct Astronomical {
    pub sunrise_time: Option<DateTime<Utc>>,
    pub sunset_time: Option<DateTime<Utc>>,
}

// #[derive(Deserialize, Debug)]
// pub struct FireDangerCategory {
//     pub text: Option<String>,
//     pub default_colour: Option<String>,
//     pub dark_mode_colour: Option<String>,
// }

// #[derive(Deserialize, Debug)]
// pub struct Now {
//     pub is_night: Option<bool>,
//     pub now_label: Option<String>,
//     pub later_label: Option<String>,
//     pub temp_now: Option<i32>,
//     pub temp_later: Option<i32>,
// }

#[derive(Deserialize, Debug)]
pub struct DailyEntry {
    pub rain: Option<Rain>,
    // pub uv: Option<UV>,
    pub astronomical: Option<Astronomical>,
    pub date: Option<DateTime<Utc>>,
    #[serde(deserialize_with = "de_temp_celsius_opt")]
    pub temp_max: Option<Temperature>,
    #[serde(deserialize_with = "de_temp_celsius_opt")]
    pub temp_min: Option<Temperature>,
    // pub extended_text: Option<String>,
    // pub icon_descriptor: Option<String>,
    // pub short_text: Option<String>,
    // pub surf_danger: Option<String>,
    // pub fire_danger: Option<String>,
    // pub fire_danger_category: Option<FireDangerCategory>,
    // pub now: Option<Now>,
}

// #[derive(Deserialize, Debug)]
// pub struct DailyMetadata {
//     pub response_timestamp: DateTime<Utc>,
//     pub issue_time: DateTime<Utc>,
//     pub next_issue_time: DateTime<Utc>,
//     pub forecast_region: String,
//     pub forecast_type: String,
//     pub copyright: String,
// }

#[derive(Deserialize, Debug)]
pub struct HourlyForecast {
    pub rain: Rain,
    #[serde(deserialize_with = "de_temp_celsius")]
    pub temp: Temperature,
    #[serde(deserialize_with = "de_temp_celsius")]
    pub temp_feels_like: Temperature,
    // pub dew_point: i16,
    pub wind: Wind,
    pub relative_humidity: RelativeHumidity,
    pub uv: Option<HourlyUV>,
    // pub icon_descriptor: String,
    // pub next_three_hourly_forecast_period: DateTime<Utc>,
    pub time: DateTime<Utc>,
    pub is_night: bool,
    // pub next_forecast_period: DateTime<Utc>,
}

#[derive(Deserialize, Debug)]
pub struct HourlyForecastResponse {
    // pub metadata: HourlyMetadata,
    pub data: Vec<HourlyForecast>,
}

#[derive(Deserialize, Debug)]
pub struct DailyForecastResponse {
    // pub metadata: DailyMetadata,
    pub data: Vec<DailyEntry>,
}

#[derive(Debug, Deserialize)]
pub struct BomError {
    pub errors: Vec<ErrorDetail>,
}

#[derive(Debug, Deserialize)]
pub struct ErrorDetail {
    // pub code: String,
    // pub title: String,
    // pub status: String,
    pub detail: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// The fixture's first hourly entry, pinned so a field-mapping regression
    /// (e.g. a serde rename mismatch silently defaulting a value) is caught
    /// instead of only checking the value falls in a plausible range.
    #[test]
    fn hourly_fixture_deserializes_first_entry_exactly() {
        let json = fs::read_to_string("tests/fixtures/bom_hourly_forecast.json")
            .expect("failed to read BOM hourly forecast fixture");
        let response: HourlyForecastResponse =
            serde_json::from_str(&json).expect("fixture should deserialize");

        let first = &response.data[0];
        assert_eq!(first.temp.value, 17.0);
        assert_eq!(first.temp_feels_like.value, 15.0);
        assert_eq!(first.wind.speed_kilometre, 9);
        assert_eq!(first.wind.gust_speed_kilometre, 19);
        assert_eq!(first.relative_humidity.0, 64);
        assert_eq!(first.uv.unwrap().0, 0);
        assert_eq!(first.rain.chance, Some(30));
        assert_eq!(first.rain.amount.min, Some(0));
        assert_eq!(first.rain.amount.max, Some(1));
        assert_eq!(
            first.time,
            "2025-10-25T11:00:00Z".parse::<DateTime<Utc>>().unwrap()
        );
        assert!(first.is_night);
    }

    /// The fixture's first daily entry, pinned the same way as the hourly test above.
    #[test]
    fn daily_fixture_deserializes_first_entry_exactly() {
        let json = fs::read_to_string("tests/fixtures/bom_daily_forecast.json")
            .expect("failed to read BOM daily forecast fixture");
        let response: DailyForecastResponse =
            serde_json::from_str(&json).expect("fixture should deserialize");

        let first = &response.data[0];
        assert_eq!(first.temp_max.unwrap().value, 20.0);
        assert!(first.temp_min.is_none());
        let rain = first.rain.as_ref().unwrap();
        assert_eq!(rain.chance, Some(60));
        assert_eq!(rain.amount.min, Some(0));
        assert_eq!(rain.amount.max, Some(2));
        let astronomical = first.astronomical.unwrap();
        assert!(astronomical.sunrise_time.is_some());
        assert!(astronomical.sunset_time.is_some());
    }

    #[test]
    fn hourly_fixture_fields_are_within_domain_bounds() {
        let json = fs::read_to_string("tests/fixtures/bom_hourly_forecast.json")
            .expect("failed to read BOM hourly forecast fixture");
        let response: HourlyForecastResponse =
            serde_json::from_str(&json).expect("fixture should deserialize");

        assert_eq!(response.data.len(), 73);
        for forecast in &response.data {
            assert!(forecast.temp.value.is_finite());
            assert!(forecast.temp_feels_like.value.is_finite());
            if let Some(chance) = forecast.rain.chance {
                assert!(chance <= 100, "rain chance should be <= 100%");
            }
        }
    }

    #[test]
    fn daily_fixture_fields_are_within_domain_bounds() {
        let json = fs::read_to_string("tests/fixtures/bom_daily_forecast.json")
            .expect("failed to read BOM daily forecast fixture");
        let response: DailyForecastResponse =
            serde_json::from_str(&json).expect("fixture should deserialize");

        assert_eq!(response.data.len(), 8);
        for entry in &response.data {
            if let Some(temp_max) = &entry.temp_max {
                assert!(temp_max.value.is_finite());
            }
            if let Some(temp_min) = &entry.temp_min {
                assert!(temp_min.value.is_finite());
            }
            if let Some(chance) = entry.rain.as_ref().and_then(|r| r.chance) {
                assert!(chance <= 100, "rain chance should be <= 100%");
            }
        }
    }

    #[test]
    fn hourly_fixture_is_chronologically_ordered() {
        let json = fs::read_to_string("tests/fixtures/bom_hourly_forecast.json")
            .expect("failed to read BOM hourly forecast fixture");
        let response: HourlyForecastResponse =
            serde_json::from_str(&json).expect("fixture should deserialize");

        assert!(response.data.len() > 1);
        for i in 1..response.data.len() {
            assert!(
                response.data[i].time > response.data[i - 1].time,
                "hourly forecasts should be in chronological order"
            );
        }
    }
}
