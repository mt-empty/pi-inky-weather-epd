// #![allow(dead_code)]
use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize, Debug, Copy, PartialOrd, PartialEq, Default, Clone)]
pub struct RelativeHumidity(pub u32);

#[derive(Deserialize, Debug, Copy, PartialOrd, PartialEq, Default, Clone)]
pub struct HourlyUV(pub u32);

#[derive(Deserialize, Debug)]
pub struct Wind {
    pub speed_kilometre: u32,
    // pub speed_knot: u32,
    // pub direction: String,
    // pub gust_speed_knot: Option<u32>,
    pub gust_speed_kilometre: Option<u32>,
}

// #[derive(Deserialize, Debug)]
// pub struct Temp {
//     pub time: DateTime<Utc>,
//     pub value: i32,
// }

#[derive(Deserialize, Debug)]
pub struct HourlyMetadata {
    // pub response_timestamp: DateTime<Utc>,
    // pub issue_time: DateTime<Utc>,
    // pub observation_time: Option<DateTime<Utc>>,
    // pub copyright: String,
}

#[derive(Deserialize, Debug)]
pub struct RainAmount {
    pub min: Option<u32>,
    pub max: Option<u32>,
    // pub lower_range: Option<u32>,
    // pub upper_range: Option<u32>,
    // pub units: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Rain {
    pub amount: RainAmount,
    pub chance: Option<u32>,
    // pub chance_of_no_rain_category: Option<String>,
    // pub precipitation_amount_25_percent_chance: Option<u32>,
    // pub precipitation_amount_50_percent_chance: Option<u32>,
    // pub precipitation_amount_75_percent_chance: Option<u32>,
}

// #[derive(Deserialize, Debug)]
// pub struct UV {
//     pub category: Option<String>,
//     pub end_time: Option<DateTime<Utc>>,
//     pub max_index: Option<u32>,
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
    pub temp_max: Option<i32>,
    pub temp_min: Option<i32>,
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
    pub temp: i32,
    pub temp_feels_like: i32,
    // pub dew_point: u32,
    pub wind: Wind,
    pub relative_humidity: RelativeHumidity,
    pub uv: HourlyUV,
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
