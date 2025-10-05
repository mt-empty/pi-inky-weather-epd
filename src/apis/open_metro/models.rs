use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};

use crate::{
    apis::bom::models::{
        Astronomical, DailyEntry, DailyForecastResponse, HourlyForecast, HourlyForecastResponse,
        HourlyUV, Rain, RainAmount, RelativeHumidity, Temperature, Wind,
    },
    configs::settings::TemperatureUnit,
};
use serde::{self, Deserialize, Deserializer};

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenMeteoHourlyResponse {
    pub latitude: f32,
    pub longitude: f32,
    #[serde(rename = "generationtime_ms")]
    pub generationtime_ms: f32,
    #[serde(rename = "utc_offset_seconds")]
    pub utc_offset_seconds: i64,
    pub timezone: String,
    #[serde(rename = "timezone_abbreviation")]
    pub timezone_abbreviation: String,
    pub elevation: f32,
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

/// Maps the deserialized OpenMeteoHourlyResponse to the desired HourlyForecastResponse structure.
pub fn map_open_meteo_to_hourly_forecast(
    open_meteo_response: OpenMeteoHourlyResponse,
) -> HourlyForecastResponse {
    let mut hourly_forecasts: Vec<HourlyForecast> = Vec::new();
    let hourly_data = open_meteo_response.hourly;

    // Open-Meteo returns data in parallel arrays, so we iterate by index
    // assuming all arrays have the same length.
    let num_entries = hourly_data.time.len();

    for i in 0..num_entries {
        // Create Temperature structs
        let temp = Temperature {
            value: hourly_data.temperature_2m[i],
            unit: TemperatureUnit::C, // Assuming Celsius based on curl command
        };
        let temp_feels_like = Temperature {
            value: hourly_data.apparent_temperature[i],
            unit: TemperatureUnit::C, // Assuming Celsius based on curl command
        };

        // Create Wind struct
        let wind = Wind {
            // Convert f32 to u16, handle potential errors or clipping if values exceed u16 max
            speed_kilometre: hourly_data.wind_speed_10m[i].round() as u16,
            gust_speed_kilometre: hourly_data.wind_gusts_10m[i].round() as u16,
        };

        // Create RelativeHumidity struct
        let relative_humidity = RelativeHumidity(hourly_data.relative_humidity_2m[i]);

        // Create HourlyUV struct
        let uv = HourlyUV(hourly_data.uv_index[i].round() as u16); // Convert f32 to u16

        // Create Rain struct
        let rain_amount = RainAmount {
            // Open-Meteo provides total precipitation per hour, not min/max
            // We'll map the single value to 'max' and leave 'min' as None or 0
            min: Some(0), // Open-Meteo gives total, so min is 0 for the hour
            max: Some(hourly_data.precipitation[i].round() as u16), // Convert f32 to u16
        };
        let rain = Rain {
            amount: rain_amount,
            chance: Some(hourly_data.precipitation_probability[i]),
        };

        // Determine if it's night
        let is_night = open_meteo_response.current.is_day == 0; // 0 from API means night

        // Create the HourlyForecast entry
        let hourly_entry = HourlyForecast {
            rain,
            temp,
            temp_feels_like,
            wind,
            relative_humidity,
            uv,
            time: hourly_data.time[i],
            is_night,
        };

        hourly_forecasts.push(hourly_entry);
    }

    // Wrap the vector in the response struct
    HourlyForecastResponse {
        data: hourly_forecasts,
    }
}

pub fn map_openmeteo_to_daily_forecast(
    response: &OpenMeteoHourlyResponse,
) -> DailyForecastResponse {
    let unit = TemperatureUnit::C;

    let daily_data = response
        .daily
        .time
        .iter()
        .enumerate()
        .map(|(i, date)| {
            let temp_max = response
                .daily
                .temperature_2m_max
                .get(i)
                .copied()
                .map(|val| {
                    let temp = Temperature {
                        value: val,
                        unit: TemperatureUnit::C,
                    };
                    match unit {
                        TemperatureUnit::C => temp,
                        TemperatureUnit::F => temp.to_fahrenheit(),
                    }
                });

            let temp_min = response
                .daily
                .temperature_2m_min
                .get(i)
                .copied()
                .map(|val| {
                    let temp = Temperature {
                        value: val,
                        unit: TemperatureUnit::C,
                    };
                    match unit {
                        TemperatureUnit::C => temp,
                        TemperatureUnit::F => temp.to_fahrenheit(),
                    }
                });

            let rain = {
                let amount = RainAmount {
                    min: None,
                    max: response.daily.precipitation_sum.get(i).map(|x| *x as u16),
                };
                let chance = response.daily.precipitation_probability_max.get(i).copied();
                if amount.max.is_some() || chance.is_some() {
                    Some(Rain { amount, chance })
                } else {
                    None
                }
            };

            let astronomical = {
                let sunrise = response
                    .daily
                    .sunrise
                    .get(i)
                    .and_then(|s| s.parse::<DateTime<Utc>>().ok());
                let sunset = response
                    .daily
                    .sunset
                    .get(i)
                    .and_then(|s| s.parse::<DateTime<Utc>>().ok());
                if sunrise.is_some() || sunset.is_some() {
                    Some(Astronomical {
                        sunrise_time: sunrise,
                        sunset_time: sunset,
                    })
                } else {
                    None
                }
            };

            DailyEntry {
                date: Some(*date),
                temp_max,
                temp_min,
                rain,
                astronomical,
            }
        })
        .collect();

    DailyForecastResponse { data: daily_data }
}

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
                .map(|naive| DateTime::<Utc>::from_utc(naive, Utc))
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
            let datetime_str = format!("{}T13:00:00Z", date_str);
            // Parse the datetime string
            NaiveDateTime::parse_from_str(&datetime_str, "%Y-%m-%dT%H:%M:%SZ")
                .map(|naive| DateTime::from_utc(naive, Utc))
                .map_err(serde::de::Error::custom) // Convert parsing error to serde error
        })
        .collect(); // Collect the results into a single Result

    datetime_vec
}
