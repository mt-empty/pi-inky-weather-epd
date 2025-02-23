#![allow(dead_code)]
use crate::{utils, CONFIG};
use chrono::NaiveDateTime;
use lazy_static::lazy_static;
use serde::Deserialize;
use utils::*;

const WEATHER_PROVIDER: &str = "https://api.weather.bom.gov.au/v1/locations";
lazy_static! {
    pub static ref DAILY_FORECAST_ENDPOINT: String = format!(
        "{}/{}/forecasts/daily",
        WEATHER_PROVIDER, CONFIG.api.location
    );
    pub static ref HOURLY_FORECAST_ENDPOINT: String = format!(
        "{}/{}/forecasts/hourly",
        WEATHER_PROVIDER, CONFIG.api.location
    );
}

#[derive(Deserialize, Debug)]
pub struct Wind {
    pub speed_kilometre: f64,
    pub speed_knot: f64,
    pub direction: String,
    pub gust_speed_knot: Option<f64>,
    pub gust_speed_kilometre: Option<f64>,
}

#[derive(Deserialize, Debug)]
pub struct Temp {
    #[serde(deserialize_with = "deserialize_naive_date")]
    pub time: NaiveDateTime,
    pub value: f64,
}

#[derive(Deserialize, Debug)]
pub struct HourlyMetadata {
    #[serde(deserialize_with = "deserialize_naive_date")]
    pub response_timestamp: NaiveDateTime,
    #[serde(deserialize_with = "deserialize_naive_date")]
    pub issue_time: NaiveDateTime,
    #[serde(default, deserialize_with = "deserialize_optional_naive_date")]
    pub observation_time: Option<NaiveDateTime>,
    pub copyright: String,
}

#[derive(Deserialize, Debug)]
pub struct RainAmount {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub lower_range: Option<f64>,
    pub upper_range: Option<f64>,
    pub units: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Rain {
    pub amount: RainAmount,
    pub chance: Option<u32>,
    pub chance_of_no_rain_category: Option<String>,
    pub precipitation_amount_25_percent_chance: Option<f64>,
    pub precipitation_amount_50_percent_chance: Option<f64>,
    pub precipitation_amount_75_percent_chance: Option<f64>,
}

#[derive(Deserialize, Debug)]
pub struct UV {
    pub category: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_naive_date")]
    pub end_time: Option<NaiveDateTime>,
    pub max_index: Option<u32>,
    #[serde(deserialize_with = "deserialize_optional_naive_date")]
    pub start_time: Option<NaiveDateTime>,
}

#[derive(Deserialize, Debug, Default, Copy, Clone)]
pub struct Astronomical {
    #[serde(deserialize_with = "deserialize_optional_naive_date")]
    pub sunrise_time: Option<NaiveDateTime>,
    #[serde(deserialize_with = "deserialize_optional_naive_date")]
    pub sunset_time: Option<NaiveDateTime>,
}

#[derive(Deserialize, Debug)]
pub struct FireDangerCategory {
    pub text: Option<String>,
    pub default_colour: Option<String>,
    pub dark_mode_colour: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Now {
    pub is_night: Option<bool>,
    pub now_label: Option<String>,
    pub later_label: Option<String>,
    pub temp_now: Option<f64>,
    pub temp_later: Option<f64>,
}

#[derive(Deserialize, Debug)]
pub struct DailyEntry {
    pub rain: Option<Rain>,
    pub uv: Option<UV>,
    pub astronomical: Option<Astronomical>,
    #[serde(deserialize_with = "deserialize_optional_naive_date")]
    pub date: Option<NaiveDateTime>,
    pub temp_max: Option<f64>,
    pub temp_min: Option<f64>,
    pub extended_text: Option<String>,
    pub icon_descriptor: Option<String>,
    pub short_text: Option<String>,
    pub surf_danger: Option<String>,
    pub fire_danger: Option<String>,
    pub fire_danger_category: Option<FireDangerCategory>,
    pub now: Option<Now>,
}

#[derive(Deserialize, Debug)]
pub struct DailyMetadata {
    #[serde(deserialize_with = "deserialize_naive_date")]
    pub response_timestamp: NaiveDateTime,
    #[serde(deserialize_with = "deserialize_naive_date")]
    pub issue_time: NaiveDateTime,
    #[serde(deserialize_with = "deserialize_naive_date")]
    pub next_issue_time: NaiveDateTime,
    pub forecast_region: String,
    pub forecast_type: String,
    pub copyright: String,
}

#[derive(Deserialize, Debug)]
pub struct HourlyForecast {
    pub rain: Rain,
    pub temp: f64,
    pub temp_feels_like: f64,
    pub dew_point: f64,
    pub wind: Wind,
    pub relative_humidity: f64,
    pub uv: f64,
    pub icon_descriptor: String,
    #[serde(deserialize_with = "deserialize_naive_date")]
    pub next_three_hourly_forecast_period: NaiveDateTime,
    #[serde(deserialize_with = "deserialize_naive_date")]
    pub time: NaiveDateTime,
    pub is_night: bool,
    #[serde(deserialize_with = "deserialize_naive_date")]
    pub next_forecast_period: NaiveDateTime,
}

#[derive(Deserialize, Debug)]
pub struct HourlyForecastResponse {
    pub metadata: HourlyMetadata,
    pub data: Vec<HourlyForecast>,
}

#[derive(Deserialize, Debug)]
pub struct DailyForecastResponse {
    pub metadata: DailyMetadata,
    pub data: Vec<DailyEntry>,
}

#[derive(Debug, Deserialize)]
pub struct BomError {
    pub errors: Vec<ErrorDetail>,
}

#[derive(Debug, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub title: String,
    pub status: String,
    pub detail: String,
}
