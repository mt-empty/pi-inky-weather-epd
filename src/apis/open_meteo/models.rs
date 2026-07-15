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
    #[serde(rename = "weather_code")]
    pub weather_code: Option<Vec<u8>>,
    /// 1 if this hour has daylight, 0 at night — computed by Open-Meteo for
    /// this exact hour and location, independent of the requested timezone.
    /// `default` so a cache written before this field existed still
    /// deserializes (falls back to empty; see `into_domain`'s per-index
    /// fallback) instead of hard-failing the whole fetch.
    #[serde(rename = "is_day", default)]
    pub is_day: Vec<u16>,
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
    /// Sunrise times as NaiveDateTime, in the forecast location's timezone
    /// (requested via `timezone=auto`) — see `OpenMeteoDailyResponse::timezone`
    /// and `into_domain`, which converts these to the display timezone.
    #[serde(deserialize_with = "deserialize_vec_naive_datetime")]
    pub sunrise: Vec<NaiveDateTime>,
    /// Sunset times as NaiveDateTime, in the forecast location's timezone —
    /// see the note on `sunrise`.
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
    #[serde(rename = "weather_code")]
    pub weather_code: Option<Vec<u8>>,
}

impl OpenMeteoHourlyResponse {
    /// Maps the API response into domain models, applying the configured
    /// temperature unit.
    pub(crate) fn into_domain(
        self,
        settings: &crate::configs::settings::DashboardSettings,
    ) -> Vec<crate::domain::models::HourlyForecast> {
        use crate::domain::models::{Precipitation, Temperature as DomainTemp, Wind as DomainWind};
        use crate::logger;

        let response = self;
        let hourly_data = response.hourly;
        let num_entries = hourly_data.time.len();
        logger::debug(format!(
            "Converting {} Open-Meteo hourly entries to domain model",
            num_entries
        ));
        let unit = settings.render_options.temp_unit;

        (0..num_entries)
            .map(|i| {
                let temperature = DomainTemp::new(
                    hourly_data.temperature_2m[i],
                    crate::configs::settings::TemperatureUnit::C,
                )
                .to_unit(unit);

                let apparent_temperature = DomainTemp::new(
                    hourly_data.apparent_temperature[i],
                    crate::configs::settings::TemperatureUnit::C,
                )
                .to_unit(unit);

                let wind = DomainWind::new(
                    hourly_data.wind_speed_10m[i].round() as u16,
                    hourly_data.wind_gusts_10m[i].round() as u16,
                );

                let precipitation = Precipitation::new_with_snowfall(
                    Some(hourly_data.precipitation_probability[i]),
                    None,
                    Some(hourly_data.precipitation[i].round() as u16),
                    // Store as tenths of a cm (×10) to preserve one decimal place of precision.
                    // Plain .round() would discard 0.1–0.49 cm, silently zeroing light snow.
                    Some((hourly_data.snowfall[i] * 10.0).round() as u16),
                );

                let uv_index = hourly_data.uv_index[i].round() as u16;
                let relative_humidity = hourly_data.relative_humidity_2m[i];
                let time = hourly_data.time[i];
                // Defaults to day (not night) if absent (see `Hourly::is_day`'s
                // doc comment) — a stale pre-upgrade cache read on a network
                // failure is the only way this is empty, and it self-corrects
                // on the next successful fetch.
                let is_night = hourly_data.is_day.get(i).is_some_and(|&d| d == 0);
                let cloud_cover = hourly_data.cloud_cover[i];
                let weather_code = hourly_data
                    .weather_code
                    .as_ref()
                    .and_then(|codes| codes.get(i).copied())
                    .map(|c| {
                        crate::domain::weather_code::WmoWeatherCode::try_from(c).map_err(|_| c)
                    });

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
                    weather_code,
                }
            })
            .collect()
    }
}

/// Converts a naive datetime returned by Open-Meteo's `timezone=auto` daily
/// endpoint — which is in the forecast location's timezone, named by
/// `location_tz_name` (the response's own `timezone` field) — into the
/// configured display timezone.
///
/// Falls back to returning `naive` unconverted (with a logged warning) if
/// `location_tz_name` isn't a recognized IANA identifier, or if `naive` falls
/// in a DST gap that doesn't exist in that timezone; sunrise/sunset times
/// essentially never land in a DST transition gap in practice.
fn convert_location_local_to_display(
    naive: NaiveDateTime,
    location_tz_name: &str,
    display_tz: chrono_tz::Tz,
) -> NaiveDateTime {
    use chrono::TimeZone;

    let Ok(location_tz) = location_tz_name.parse::<chrono_tz::Tz>() else {
        crate::logger::warning(format!(
            "Open-Meteo returned unrecognized timezone '{location_tz_name}'; \
             using sunrise/sunset time unconverted"
        ));
        return naive;
    };

    match location_tz.from_local_datetime(&naive) {
        chrono::LocalResult::Single(dt) => dt.with_timezone(&display_tz).naive_local(),
        chrono::LocalResult::Ambiguous(dt, _) => dt.with_timezone(&display_tz).naive_local(),
        chrono::LocalResult::None => {
            crate::logger::warning(format!(
                "Sunrise/sunset time {naive} does not exist in timezone \
                 '{location_tz_name}' (DST gap); using unconverted"
            ));
            naive
        }
    }
}

impl OpenMeteoDailyResponse {
    /// Maps the API response into domain models, applying the configured
    /// temperature unit.
    pub(crate) fn into_domain(
        self,
        settings: &crate::configs::settings::DashboardSettings,
    ) -> Vec<crate::domain::models::DailyForecast> {
        use crate::domain::models::{Astronomical, Precipitation, Temperature as DomainTemp};
        use crate::logger;

        let response = self;
        let unit = settings.render_options.temp_unit;
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
                let temp_max = Some(
                    DomainTemp::new(
                        response.daily.temperature_2m_max[i],
                        crate::configs::settings::TemperatureUnit::C,
                    )
                    .to_unit(unit),
                );

                let temp_min = Some(
                    DomainTemp::new(
                        response.daily.temperature_2m_min[i],
                        crate::configs::settings::TemperatureUnit::C,
                    )
                    .to_unit(unit),
                );

                let precipitation = {
                    let amount_max = response.daily.precipitation_sum[i].round() as u16;
                    let chance = response.daily.precipitation_probability_max[i];
                    // Store as tenths of a cm (×10) — same convention as hourly snowfall.
                    let snowfall_amount = (response.daily.snowfall_sum[i] * 10.0).round() as u16;

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
                    // sunrise/sunset arrive in the forecast location's timezone
                    // (response.timezone, from timezone=auto); convert to the
                    // display timezone so they agree with every other rendered time.
                    let sunrise = response.daily.sunrise.get(i).copied().map(|dt| {
                        convert_location_local_to_display(
                            dt,
                            &response.timezone,
                            settings.misc.timezone,
                        )
                    });
                    let sunset = response.daily.sunset.get(i).copied().map(|dt| {
                        convert_location_local_to_display(
                            dt,
                            &response.timezone,
                            settings.misc.timezone,
                        )
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

                let cloud_cover = response.daily.cloud_cover_mean.get(i).and_then(|&c| c);
                let weather_code = response
                    .daily
                    .weather_code
                    .as_ref()
                    .and_then(|codes| codes.get(i).copied())
                    .map(|c| {
                        crate::domain::weather_code::WmoWeatherCode::try_from(c).map_err(|_| c)
                    });

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
                    weather_code,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// The fixture's first hourly entry, pinned so a field-mapping regression
    /// (e.g. a serde rename mismatch silently defaulting a value) is caught
    /// instead of only checking the value falls in a plausible range.
    #[test]
    fn hourly_fixture_deserializes_first_entry_exactly() {
        let json = fs::read_to_string("tests/fixtures/open_meteo_hourly_forecast.json")
            .expect("failed to read Open-Meteo hourly forecast fixture");
        let response: OpenMeteoHourlyResponse =
            serde_json::from_str(&json).expect("fixture should deserialize");

        assert_eq!(response.latitude, -37.75);
        assert_eq!(response.longitude, 144.875);
        assert_eq!(response.hourly.time.len(), 168);
        assert_eq!(response.hourly.temperature_2m[0], 15.5);
        assert_eq!(response.hourly.apparent_temperature[0], 14.3);
        assert_eq!(response.hourly.precipitation_probability[0], 0);
        assert_eq!(response.hourly.precipitation[0], 0.0);
        assert_eq!(response.hourly.uv_index[0], 4.6);
        assert_eq!(response.hourly.wind_speed_10m[0], 4.1);
        assert_eq!(response.hourly.wind_gusts_10m[0], 14.0);
        assert_eq!(response.hourly.relative_humidity_2m[0], 61);
        assert_eq!(response.hourly.cloud_cover[0], Some(1));
        assert_eq!(response.hourly.snowfall[0], 0.0);
        assert_eq!(response.hourly.weather_code.as_ref().unwrap()[0], 3);
    }

    /// The fixture's first daily entry, pinned the same way as the hourly test above.
    #[test]
    fn daily_fixture_deserializes_first_entry_exactly() {
        let json = fs::read_to_string("tests/fixtures/open_meteo_daily_forecast.json")
            .expect("failed to read Open-Meteo daily forecast fixture");
        let response: OpenMeteoDailyResponse =
            serde_json::from_str(&json).expect("fixture should deserialize");

        assert_eq!(response.daily.time.len(), 7);
        assert_eq!(response.daily.temperature_2m_max[0], 18.5);
        assert_eq!(response.daily.temperature_2m_min[0], 13.1);
        assert_eq!(response.daily.precipitation_sum[0], 6.6);
        assert_eq!(response.daily.precipitation_probability_max[0], 93);
        assert_eq!(response.daily.cloud_cover_mean[0], Some(88));
        assert_eq!(response.daily.snowfall_sum[0], 0.0);
        assert_eq!(response.daily.weather_code.as_ref().unwrap()[0], 53);
    }

    #[test]
    fn hourly_snowfall_array_present_and_aligned() {
        let json = fs::read_to_string("tests/fixtures/open_meteo_hourly_forecast.json")
            .expect("failed to read Open-Meteo hourly forecast fixture");
        let response: OpenMeteoHourlyResponse =
            serde_json::from_str(&json).expect("fixture should deserialize");

        assert_eq!(response.hourly.snowfall.len(), response.hourly.time.len());
        assert!(
            response.hourly.snowfall.iter().any(|&s| s > 0.0),
            "fixture should contain at least some snowfall data"
        );
    }

    #[test]
    fn daily_snowfall_sum_present_and_aligned() {
        let json = fs::read_to_string("tests/fixtures/open_meteo_daily_forecast.json")
            .expect("failed to read Open-Meteo daily forecast fixture");
        let response: OpenMeteoDailyResponse =
            serde_json::from_str(&json).expect("fixture should deserialize");

        assert_eq!(response.daily.snowfall_sum.len(), response.daily.time.len());
        assert!(
            response.daily.snowfall_sum.iter().any(|&s| s > 0.0),
            "fixture should contain at least some daily snowfall data"
        );
    }

    #[test]
    fn hourly_fixture_fields_are_within_domain_bounds() {
        let json = fs::read_to_string("tests/fixtures/open_meteo_hourly_forecast.json")
            .expect("failed to read Open-Meteo hourly forecast fixture");
        let response: OpenMeteoHourlyResponse =
            serde_json::from_str(&json).expect("fixture should deserialize");
        let hourly = &response.hourly;

        for i in 0..hourly.time.len() {
            assert!(hourly.temperature_2m[i] > -50.0 && hourly.temperature_2m[i] < 60.0);
            assert!(hourly.apparent_temperature[i].is_finite());
            assert!(hourly.precipitation_probability[i] <= 100);
            assert!((0.0..500.0).contains(&hourly.precipitation[i]));
            assert!((0.0..20.0).contains(&hourly.uv_index[i]));
            assert!((0.0..500.0).contains(&hourly.wind_speed_10m[i]));
            assert!(hourly.relative_humidity_2m[i] <= 100);
        }
    }

    #[test]
    fn daily_fixture_fields_are_within_domain_bounds() {
        let json = fs::read_to_string("tests/fixtures/open_meteo_daily_forecast.json")
            .expect("failed to read Open-Meteo daily forecast fixture");
        let response: OpenMeteoDailyResponse =
            serde_json::from_str(&json).expect("fixture should deserialize");
        let daily = &response.daily;

        for i in 0..daily.time.len() {
            let temp_max = daily.temperature_2m_max[i];
            let temp_min = daily.temperature_2m_min[i];
            assert!(temp_max > -50.0 && temp_max < 60.0);
            assert!(temp_min > -50.0 && temp_min < 60.0);
            assert!(temp_max >= temp_min);
            assert!((0.0..500.0).contains(&daily.precipitation_sum[i]));
            assert!(daily.precipitation_probability_max[i] <= 100);
        }
    }

    #[test]
    fn hourly_fixture_is_chronologically_ordered() {
        let json = fs::read_to_string("tests/fixtures/open_meteo_hourly_forecast.json")
            .expect("failed to read Open-Meteo hourly forecast fixture");
        let response: OpenMeteoHourlyResponse =
            serde_json::from_str(&json).expect("fixture should deserialize");
        let hourly = &response.hourly;

        assert!(hourly.time.len() > 1);
        for i in 1..hourly.time.len() {
            assert!(hourly.time[i] > hourly.time[i - 1]);
        }
    }

    #[test]
    fn daily_fixture_is_chronologically_ordered() {
        let json = fs::read_to_string("tests/fixtures/open_meteo_daily_forecast.json")
            .expect("failed to read Open-Meteo daily forecast fixture");
        let response: OpenMeteoDailyResponse =
            serde_json::from_str(&json).expect("fixture should deserialize");
        let daily = &response.daily;

        assert!(daily.time.len() > 1);
        for i in 1..daily.time.len() {
            assert!(daily.time[i] > daily.time[i - 1]);
        }
    }

    #[test]
    fn coordinates_are_within_valid_range() {
        let json = fs::read_to_string("tests/fixtures/open_meteo_hourly_forecast.json")
            .expect("failed to read Open-Meteo hourly forecast fixture");
        let response: OpenMeteoHourlyResponse =
            serde_json::from_str(&json).expect("fixture should deserialize");

        assert!(response.latitude >= -90.0 && response.latitude <= 90.0);
        assert!(response.longitude >= -180.0 && response.longitude <= 180.0);
    }

    /// Verifies date-to-day-name conversion works correctly for the daily response
    #[test]
    fn daily_dates_deserialize_correctly() {
        use chrono::{Datelike, Weekday};

        let json = r#"{
            "latitude":-37.75,
            "longitude":144.875,
            "timezone":"GMT",
            "timezone_abbreviation":"GMT",
            "current_units":{"time":"iso8601","interval":"seconds","is_day":""},
            "current":{"time":"2025-10-25T12:00","interval":900,"is_day":1},
            "hourly_units":{"time":"iso8601","temperature_2m":"°C","apparent_temperature":"°C","precipitation_probability":"%","precipitation":"mm","snowfall":"cm","uv_index":"","wind_speed_10m":"km/h","wind_gusts_10m":"km/h","relative_humidity_2m":"%"},
            "hourly":{"time":["2025-10-25T12:00"],"temperature_2m":[20.0],"apparent_temperature":[18.0],"precipitation_probability":[10],"precipitation":[0.0],"snowfall":[0.0],"uv_index":[5.0],"wind_speed_10m":[15.0],"wind_gusts_10m":[25.0],"relative_humidity_2m":[50],"cloud_cover":[30]},
            "daily_units":{"time":"iso8601","sunrise":"iso8601","sunset":"iso8601","temperature_2m_max":"°C","temperature_2m_min":"°C","precipitation_sum":"mm","precipitation_probability_max":"%","snowfall_sum":"cm"},
            "daily":{"time":["2025-10-25","2025-10-26","2025-10-27","2025-10-28","2025-10-29","2025-10-30","2025-10-31"],"sunrise":["2025-10-25T19:00","2025-10-26T19:00","2025-10-27T19:00","2025-10-28T19:00","2025-10-29T19:00","2025-10-30T19:00","2025-10-31T19:00"],"sunset":["2025-10-25T09:00","2025-10-26T09:00","2025-10-27T09:00","2025-10-28T09:00","2025-10-29T09:00","2025-10-30T09:00","2025-10-31T09:00"],"temperature_2m_max":[22.0,23.0,24.0,25.0,26.0,27.0,28.0],"temperature_2m_min":[12.0,13.0,14.0,15.0,16.0,17.0,18.0],"precipitation_sum":[0.0,1.0,2.0,0.0,0.0,0.0,0.0],"precipitation_probability_max":[10,30,50,20,10,5,0],"snowfall_sum":[0.0,0.0,0.0,0.0,0.0,0.0,0.0],"cloud_cover_mean":[20,45,65,25,10,8,5]}
        }"#;

        let response: OpenMeteoDailyResponse =
            serde_json::from_str(json).expect("inline fixture should deserialize");

        // October 25, 2025 is a Saturday
        let expected_days = [
            (Weekday::Sat, "2025-10-25"),
            (Weekday::Sun, "2025-10-26"),
            (Weekday::Mon, "2025-10-27"),
            (Weekday::Tue, "2025-10-28"),
            (Weekday::Wed, "2025-10-29"),
            (Weekday::Thu, "2025-10-30"),
            (Weekday::Fri, "2025-10-31"),
        ];

        assert_eq!(response.daily.time.len(), 7);
        for (i, (expected_weekday, expected_date_str)) in expected_days.iter().enumerate() {
            let date = response.daily.time[i];
            assert_eq!(&date.format("%Y-%m-%d").to_string(), expected_date_str);
            assert_eq!(&date.weekday(), expected_weekday);
        }
    }
}
