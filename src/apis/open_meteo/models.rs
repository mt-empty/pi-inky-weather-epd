use chrono::{DateTime, NaiveDateTime, Utc};
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
    // #[serde(rename = "generationtime_ms")]
    // pub generationtime_ms: f32,
    // #[serde(rename = "utc_offset_seconds")]
    // pub utc_offset_seconds: i64,
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
    // pub time: DateTime<Utc>,
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
    #[serde(rename = "uv_index")]
    pub uv_index: Vec<f32>,
    #[serde(rename = "wind_speed_10m")]
    pub wind_speed_10m: Vec<f32>,
    #[serde(rename = "wind_gusts_10m")]
    pub wind_gusts_10m: Vec<f32>,
    #[serde(rename = "relative_humidity_2m")]
    pub relative_humidity_2m: Vec<u16>,
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
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Daily {
    #[serde(deserialize_with = "deserialize_vec_daily_datetime")]
    pub time: Vec<DateTime<Utc>>,
    // #[serde(deserialize_with = "deserialize_vec_iso8601_loose")]
    pub sunrise: Vec<String>,
    // #[serde(deserialize_with = "deserialize_vec_iso8601_loose")]
    pub sunset: Vec<String>,
    #[serde(rename = "temperature_2m_max")]
    pub temperature_2m_max: Vec<f32>,
    #[serde(rename = "temperature_2m_min")]
    pub temperature_2m_min: Vec<f32>,
    #[serde(rename = "precipitation_sum")]
    pub precipitation_sum: Vec<f32>,
    #[serde(rename = "precipitation_probability_max")]
    pub precipitation_probability_max: Vec<u16>,
}

impl From<OpenMeteoHourlyResponse> for Vec<crate::domain::models::HourlyForecast> {
    fn from(response: OpenMeteoHourlyResponse) -> Self {
        use crate::domain::models::{Precipitation, Temperature as DomainTemp, Wind as DomainWind};
        use crate::CONFIG;

        let hourly_data = response.hourly;
        let num_entries = hourly_data.time.len();
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

                let precipitation = Precipitation::new(
                    Some(hourly_data.precipitation_probability[i]),
                    None,
                    Some(hourly_data.precipitation[i].round() as u16),
                );

                let uv_index = hourly_data.uv_index[i].round() as u16;
                let relative_humidity = hourly_data.relative_humidity_2m[i];
                let time = hourly_data.time[i];
                let is_night = response.current.is_day == 0;

                crate::domain::models::HourlyForecast {
                    time,
                    temperature,
                    apparent_temperature,
                    wind,
                    precipitation,
                    uv_index,
                    relative_humidity,
                    is_night,
                }
            })
            .collect()
    }
}

impl From<OpenMeteoHourlyResponse> for Vec<crate::domain::models::DailyForecast> {
    fn from(response: OpenMeteoHourlyResponse) -> Self {
        use crate::domain::models::{Astronomical, Precipitation, Temperature as DomainTemp};
        use crate::CONFIG;

        let unit = CONFIG.render_options.temp_unit;

        response
            .daily
            .time
            .iter()
            .enumerate()
            .map(|(i, date)| {
                let temp_max = {
                    let val = response.daily.temperature_2m_max[i];
                    let temp = DomainTemp::new(val, crate::configs::settings::TemperatureUnit::C);
                    Some(match unit {
                        crate::configs::settings::TemperatureUnit::C => temp,
                        crate::configs::settings::TemperatureUnit::F => temp.to_fahrenheit(),
                    })
                };

                let temp_min = {
                    let val = response.daily.temperature_2m_min[i];
                    let temp = DomainTemp::new(val, crate::configs::settings::TemperatureUnit::C);
                    Some(match unit {
                        crate::configs::settings::TemperatureUnit::C => temp,
                        crate::configs::settings::TemperatureUnit::F => temp.to_fahrenheit(),
                    })
                };

                let precipitation = {
                    let amount_max = response.daily.precipitation_sum[i].round() as u16;
                    let chance = response.daily.precipitation_probability_max[i];

                    if amount_max > 0 || chance > 0 {
                        Some(Precipitation::new(Some(chance), None, Some(amount_max)))
                    } else {
                        None
                    }
                };

                let astronomical = {
                    let sunrise = response.daily.sunrise.get(i).and_then(|s| {
                        chrono::DateTime::parse_from_rfc3339(s)
                            .ok()
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                    });
                    let sunset = response.daily.sunset.get(i).and_then(|s| {
                        chrono::DateTime::parse_from_rfc3339(s)
                            .ok()
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                    });

                    if sunrise.is_some() || sunset.is_some() {
                        Some(Astronomical {
                            sunrise_time: sunrise,
                            sunset_time: sunset,
                        })
                    } else {
                        None
                    }
                };

                crate::domain::models::DailyForecast {
                    date: Some(*date),
                    temp_max,
                    temp_min,
                    precipitation,
                    astronomical,
                }
            })
            .collect()
    }
}

// ============================================================================
// Custom deserializers for OpenMeteo date/time formats
// ============================================================================
pub fn deserialize_vec_iso8601_loose<'de, D>(
    deserializer: D,
) -> Result<Vec<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw_vec: Vec<String> = Deserialize::deserialize(deserializer)?;
    raw_vec
        .into_iter()
        .map(|mut s| {
            // If seconds missing, add ":00"
            if s.len() == 16 {
                s.push_str(":00");
            }
            // If timezone missing, assume UTC
            if !s.ends_with('Z') && !s.contains('+') && !s.contains('-') {
                s.push('Z');
            }

            DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(serde::de::Error::custom)
        })
        .collect()
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

pub fn deserialize_vec_daily_datetime<'de, D>(
    deserializer: D,
) -> Result<Vec<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    // Deserialize the input as a vector of strings
    let date_strings: Vec<String> = Deserialize::deserialize(deserializer)?;

    // Map the date strings to DateTime<Utc> with the fixed time
    let datetime_vec: Result<Vec<DateTime<Utc>>, D::Error> = date_strings
        .into_iter()
        .map(|date_str| {
            // Combine the date string with the fixed time
            let datetime_str = format!("{date_str}T13:00:00Z");
            // Parse the datetime string
            NaiveDateTime::parse_from_str(&datetime_str, "%Y-%m-%dT%H:%M:%SZ")
                .map(|naive| DateTime::from_naive_utc_and_offset(naive, Utc))
                .map_err(serde::de::Error::custom) // Convert parsing error to serde error
        })
        .collect(); // Collect the results into a single Result

    datetime_vec
}
