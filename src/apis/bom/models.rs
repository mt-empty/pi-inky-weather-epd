// #![allow(dead_code)]
use super::utils::*;
use std::{
    fmt::{self, Display},
    ops::Deref,
};

use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::{configs::settings::TemperatureUnit, CONFIG};

#[derive(Deserialize, Debug, Copy, PartialOrd, PartialEq, Default, Clone)]
pub struct RelativeHumidity(pub u16);

#[derive(Deserialize, Debug, Copy, PartialOrd, PartialEq, Default, Clone)]
pub struct HourlyUV(pub u16);

#[derive(Deserialize, Debug, PartialOrd, PartialEq, Clone, Copy)]
pub struct Temperature {
    pub value: f32,
    pub unit: TemperatureUnit,
}

impl Temperature {
    pub fn to_celsius(self) -> Temperature {
        match self.unit {
            TemperatureUnit::C => self,
            TemperatureUnit::F => Temperature {
                value: (self.value - 32.0) * 5.0 / 9.0,
                unit: TemperatureUnit::C,
            },
        }
    }
    pub fn to_fahrenheit(self) -> Temperature {
        match self.unit {
            TemperatureUnit::C => Temperature {
                value: (self.value * 9.0 / 5.0) + 32.0,
                unit: TemperatureUnit::F,
            },
            TemperatureUnit::F => self,
        }
    }
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

impl Wind {
    pub fn get_speed(&self) -> u16 {
        if CONFIG.render_options.use_gust_instead_of_wind {
            self.gust_speed_kilometre
        } else {
            self.speed_kilometre
        }
    }
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
