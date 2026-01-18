use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use serde::{self, Deserialize, Deserializer};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct OpenMeteoError {
    pub error: bool,
    pub reason: String,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenMeteoHourlyResponse {
    pub latitude: f32,
    pub longitude: f32,
    pub timezone: String,
    // #[serde(rename = "timezone_abbreviation")]
    // pub timezone_abbreviation: String,
    // pub elevation: f32,
    #[serde(rename = "current_units")]
    pub current_units: CurrentUnits,
    pub current: Current,
    #[serde(rename = "hourly_units")]
    pub hourly_units: HourlyUnits,
    pub hourly: Hourly,
}

/// Response from Open-Meteo API for daily forecast data
///
/// This is requested separately from hourly data with timezone-specific parameters
/// to ensure daily aggregations (max/min temp, precipitation totals) represent
/// the user's local 24-hour window, not UTC's 24-hour window.
#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenMeteoDailyResponse {
    pub latitude: f32,
    pub longitude: f32,
    pub timezone: String,
    #[serde(rename = "daily_units")]
    pub daily_units: DailyUnits,
    pub daily: Daily,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentUnits {
    // pub time: DateTime<Utc>,
    pub interval: String,
    #[serde(rename = "is_day")]
    pub is_day: String,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Current {
    #[serde(deserialize_with = "deserialize_short_datetime")]
    pub time: DateTime<Utc>,
    // pub interval: i64,
    #[serde(rename = "is_day")]
    pub is_day: u16,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HourlyUnits {
    // pub time: DateTime<Utc>,
    #[serde(rename = "temperature_2m")]
    pub temperature_2m: String,
    #[serde(rename = "apparent_temperature")]
    pub apparent_temperature: String,
    #[serde(rename = "precipitation_probability")]
    pub precipitation_probability: String,
    pub precipitation: String,
    pub snowfall: String,
    #[serde(rename = "uv_index")]
    pub uv_index: String,
    #[serde(rename = "wind_speed_10m")]
    pub wind_speed_10m: String,
    #[serde(rename = "wind_gusts_10m")]
    pub wind_gusts_10m: String,
    #[serde(rename = "relative_humidity_2m")]
    pub relative_humidity_2m: String,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hourly {
    #[serde(deserialize_with = "deserialize_vec_short_datetime")]
    pub time: Vec<DateTime<Utc>>,
    #[serde(rename = "temperature_2m")]
    pub temperature_2m: Vec<f32>,
    #[serde(rename = "apparent_temperature")]
    pub apparent_temperature: Vec<f32>,
    #[serde(rename = "precipitation_probability")]
    pub precipitation_probability: Vec<u16>,
    pub precipitation: Vec<f32>,
    pub snowfall: Vec<f32>,
    #[serde(rename = "uv_index")]
    pub uv_index: Vec<f32>,
    #[serde(rename = "wind_speed_10m")]
    pub wind_speed_10m: Vec<f32>,
    #[serde(rename = "wind_gusts_10m")]
    pub wind_gusts_10m: Vec<f32>,
    #[serde(rename = "relative_humidity_2m")]
    pub relative_humidity_2m: Vec<u16>,
    #[serde(rename = "cloud_cover")]
    pub cloud_cover: Vec<Option<u16>>,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyUnits {
    // pub time: DateTime<Utc>,
    // pub sunrise: String,
    // pub sunset: String,
    #[serde(rename = "temperature_2m_max")]
    pub temperature_2m_max: String,
    #[serde(rename = "temperature_2m_min")]
    pub temperature_2m_min: String,
    #[serde(rename = "precipitation_sum")]
    pub precipitation_sum: String,
    #[serde(rename = "precipitation_probability_max")]
    pub precipitation_probability_max: String,
    #[serde(rename = "snowfall_sum")]
    pub snowfall_sum: String,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Daily {
    pub time: Vec<NaiveDate>,
    /// Sunrise times as NaiveDateTime (timezone-agnostic)
    /// When timezone=auto is used, these represent local time and must be converted using response.timezone
    #[serde(deserialize_with = "deserialize_vec_naive_datetime")]
    pub sunrise: Vec<NaiveDateTime>,
    /// Sunset times as NaiveDateTime (timezone-agnostic)
    /// When timezone=auto is used, these represent local time and must be converted using response.timezone
    #[serde(deserialize_with = "deserialize_vec_naive_datetime")]
    pub sunset: Vec<NaiveDateTime>,
    #[serde(rename = "temperature_2m_max")]
    pub temperature_2m_max: Vec<f32>,
    #[serde(rename = "temperature_2m_min")]
    pub temperature_2m_min: Vec<f32>,
    #[serde(rename = "precipitation_sum")]
    pub precipitation_sum: Vec<f32>,
    #[serde(rename = "precipitation_probability_max")]
    pub precipitation_probability_max: Vec<u16>,
    #[serde(rename = "snowfall_sum")]
    pub snowfall_sum: Vec<f32>,
    #[serde(rename = "cloud_cover_mean")]
    pub cloud_cover_mean: Vec<Option<u16>>,
}

impl From<OpenMeteoHourlyResponse> for Vec<crate::domain::models::HourlyForecast> {
    fn from(response: OpenMeteoHourlyResponse) -> Self {
        use crate::domain::models::{Precipitation, Temperature as DomainTemp, Wind as DomainWind};
        use crate::{logger, CONFIG};

        let hourly_data = response.hourly;
        let num_entries = hourly_data.time.len();
        logger::debug(format!(
            "Converting {} Open-Meteo hourly entries to domain model",
            num_entries
        ));
        let unit = CONFIG.render_options.temp_unit;

        (0..num_entries)
            .map(|i| {
                let temperature = {
                    let val = hourly_data.temperature_2m[i];
                    let temp = DomainTemp::new(val, crate::configs::settings::TemperatureUnit::C);
                    match unit {
                        crate::configs::settings::TemperatureUnit::C => temp,
                        crate::configs::settings::TemperatureUnit::F => temp.to_fahrenheit(),
                    }
                };

                let apparent_temperature = {
                    let val = hourly_data.apparent_temperature[i];
                    let temp = DomainTemp::new(val, crate::configs::settings::TemperatureUnit::C);
                    match unit {
                        crate::configs::settings::TemperatureUnit::C => temp,
                        crate::configs::settings::TemperatureUnit::F => temp.to_fahrenheit(),
                    }
                };

                let wind = DomainWind::new(
                    hourly_data.wind_speed_10m[i].round() as u16,
                    hourly_data.wind_gusts_10m[i].round() as u16,
                );

                let precipitation = Precipitation::new_with_snowfall(
                    Some(hourly_data.precipitation_probability[i]),
                    None,
                    Some(hourly_data.precipitation[i].round() as u16),
                    Some(hourly_data.snowfall[i].round() as u16),
                );

                let uv_index = hourly_data.uv_index[i].round() as u16;
                let relative_humidity = hourly_data.relative_humidity_2m[i];
                let time = hourly_data.time[i];
                let is_night = response.current.is_day == 0;
                let cloud_cover = hourly_data.cloud_cover[i];

                crate::domain::models::HourlyForecast {
                    time,
                    temperature,
                    apparent_temperature,
                    wind,
                    precipitation,
                    uv_index,
                    relative_humidity,
                    is_night,
                    cloud_cover,
                }
            })
            .collect()
    }
}

impl From<OpenMeteoDailyResponse> for Vec<crate::domain::models::DailyForecast> {
    fn from(response: OpenMeteoDailyResponse) -> Self {
        use crate::domain::models::{Astronomical, Precipitation, Temperature as DomainTemp};
        use crate::{logger, CONFIG};

        let unit = CONFIG.render_options.temp_unit;
        logger::debug(format!(
            "Converting {} Open-Meteo daily entries to domain model",
            response.daily.time.len()
        ));

        response
            .daily
            .time
            .iter()
            .enumerate()
            .map(|(i, date)| {
                let raw_temp_max = response.daily.temperature_2m_max[i];
                let raw_temp_min = response.daily.temperature_2m_min[i];

                let temp_max = {
                    let val = raw_temp_max;
                    let temp = DomainTemp::new(val, crate::configs::settings::TemperatureUnit::C);
                    Some(match unit {
                        crate::configs::settings::TemperatureUnit::C => temp,
                        crate::configs::settings::TemperatureUnit::F => temp.to_fahrenheit(),
                    })
                };

                let temp_min = {
                    let val = raw_temp_min;
                    let temp = DomainTemp::new(val, crate::configs::settings::TemperatureUnit::C);
                    Some(match unit {
                        crate::configs::settings::TemperatureUnit::C => temp,
                        crate::configs::settings::TemperatureUnit::F => temp.to_fahrenheit(),
                    })
                };

                let precipitation = {
                    let amount_max = response.daily.precipitation_sum[i].round() as u16;
                    let chance = response.daily.precipitation_probability_max[i];
                    let snowfall_amount = response.daily.snowfall_sum[i].round() as u16;

                    if amount_max > 0 || chance > 0 || snowfall_amount > 0 {
                        Some(Precipitation::new_with_snowfall(
                            Some(chance),
                            None,
                            Some(amount_max),
                            Some(snowfall_amount),
                        ))
                    } else {
                        None
                    }
                };

                let astronomical = {
                    // With timezone=auto, sunrise/sunset are already in local time (NaiveDateTime)
                    // Store them directly without timezone conversion
                    let sunrise = response.daily.sunrise.get(i).copied();
                    let sunset = response.daily.sunset.get(i).copied();

                    if sunrise.is_some() || sunset.is_some() {
                        Some(Astronomical {
                            sunrise_time: sunrise,
                            sunset_time: sunset,
                        })
                    } else {
                        None
                    }
                };

                let cloud_cover = response.daily.cloud_cover_mean.get(i).and_then(|&c| c);

                crate::domain::models::DailyForecast {
                    // Use NaiveDate directly - API returns dates in user's local timezone
                    // When timezone=America/New_York, "2025-12-28" = Dec 28 in NY time
                    // Daily aggregations (max/min) are computed over NY's 24-hour window
                    date: Some(*date),
                    temp_max,
                    temp_min,
                    precipitation,
                    astronomical,
                    cloud_cover,
                }
            })
            .collect()
    }
}

// ============================================================================
// Custom deserializers for OpenMeteo date/time formats
// ============================================================================

/// Deserializes a vector of datetime strings to NaiveDateTime (no timezone)
/// Used for sunrise/sunset times when timezone=auto, which returns local times
pub fn deserialize_vec_naive_datetime<'de, D>(
    deserializer: D,
) -> Result<Vec<NaiveDateTime>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw_vec: Vec<String> = Deserialize::deserialize(deserializer)?;
    raw_vec
        .into_iter()
        .map(|s| {
            NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M").map_err(serde::de::Error::custom)
        })
        .collect()
}

/// Deserializes datetime string for hourly data (always UTC)
pub fn deserialize_short_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M")
        .map(|naive| DateTime::from_naive_utc_and_offset(naive, Utc))
        .map_err(serde::de::Error::custom)
}

pub fn deserialize_vec_iso8601_loose<'de, D>(
    deserializer: D,
) -> Result<Vec<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_vec_short_datetime(deserializer)
}

pub fn deserialize_vec_short_datetime<'de, D>(
    deserializer: D,
) -> Result<Vec<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw_vec: Vec<String> = Deserialize::deserialize(deserializer)?;
    raw_vec
        .into_iter()
        .map(|s| {
            NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M")
                .map(|naive| DateTime::from_naive_utc_and_offset(naive, Utc))
                .map_err(serde::de::Error::custom)
        })
        .collect()
}
