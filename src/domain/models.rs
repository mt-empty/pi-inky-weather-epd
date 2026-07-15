use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use std::{
    fmt::{self, Display},
    ops::Deref,
};

use crate::configs::settings::TemperatureUnit;
use crate::domain::weather_code::WmoWeatherCode;

/// Domain-specific Temperature type, independent of any API
#[derive(Debug, Copy, PartialOrd, PartialEq, Clone)]
pub struct Temperature {
    pub value: f32,
    pub unit: TemperatureUnit,
}

impl Temperature {
    pub fn new(value: f32, unit: TemperatureUnit) -> Self {
        Self { value, unit }
    }

    pub fn celsius(value: f32) -> Self {
        Self {
            value,
            unit: TemperatureUnit::C,
        }
    }

    pub fn fahrenheit(value: f32) -> Self {
        Self {
            value,
            unit: TemperatureUnit::F,
        }
    }

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

    /// Converts to the given unit, dispatching to `to_celsius`/`to_fahrenheit`.
    pub fn to_unit(self, unit: TemperatureUnit) -> Temperature {
        match unit {
            TemperatureUnit::C => self.to_celsius(),
            TemperatureUnit::F => self.to_fahrenheit(),
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

/// Convert from BOM Temperature to domain Temperature
impl From<crate::apis::bom::models::Temperature> for Temperature {
    fn from(bom_temp: crate::apis::bom::models::Temperature) -> Self {
        Temperature {
            value: bom_temp.value,
            unit: bom_temp.unit,
        }
    }
}

/// Domain model for wind information
#[derive(Debug, Clone)]
pub struct Wind {
    pub speed_kmh: u16,
    pub gust_speed_kmh: u16,
}

impl Wind {
    pub fn new(speed_kmh: u16, gust_speed_kmh: u16) -> Self {
        Self {
            speed_kmh,
            gust_speed_kmh,
        }
    }

    pub fn speed(&self, use_gust: bool) -> u16 {
        if use_gust {
            self.gust_speed_kmh
        } else {
            self.speed_kmh
        }
    }

    /// Convert wind speed from km/h to the specified unit
    pub fn convert_speed(speed_kmh: u16, unit: crate::configs::settings::WindSpeedUnit) -> u16 {
        use crate::configs::settings::WindSpeedUnit;
        match unit {
            WindSpeedUnit::KmH => speed_kmh,
            WindSpeedUnit::Mph => (speed_kmh as f64 * 0.621371).round() as u16,
            WindSpeedUnit::Knots => (speed_kmh as f64 * 0.539957).round() as u16,
        }
    }

    /// Get the wind speed in the configured unit
    pub fn speed_in_unit(
        &self,
        use_gust: bool,
        unit: crate::configs::settings::WindSpeedUnit,
    ) -> u16 {
        let speed_kmh = self.speed(use_gust);
        Self::convert_speed(speed_kmh, unit)
    }
}

/// Domain model for precipitation information
#[derive(Debug, Clone)]
pub struct Precipitation {
    pub chance: Option<u16>,
    pub amount_min: Option<u16>,
    pub amount_max: Option<u16>,
    /// Snowfall in **tenths of a centimetre** (×10).
    ///
    /// Stored at sub-cm precision to avoid rounding out light snowfall (0.1–0.49 cm).
    /// Always divide by 10 before use — prefer `snowfall_cm()` over reading this field directly.
    /// Open-Meteo unit: cm; multiply by 10 before storing, divide by 10 when reading.
    pub snowfall_amount: Option<u16>,
}

impl Precipitation {
    pub fn new(chance: Option<u16>, amount_min: Option<u16>, amount_max: Option<u16>) -> Self {
        Self {
            chance,
            amount_min,
            amount_max,
            snowfall_amount: None,
        }
    }
    // TODO: should use a single new with optional snowfall
    pub fn new_with_snowfall(
        chance: Option<u16>,
        amount_min: Option<u16>,
        amount_max: Option<u16>,
        snowfall_amount: Option<u16>,
    ) -> Self {
        Self {
            chance,
            amount_min,
            amount_max,
            snowfall_amount,
        }
    }

    pub fn median(&self) -> f32 {
        let min = self.amount_min.unwrap_or(0);
        let max = self.amount_max.unwrap_or(min);
        (min + max) as f32 / 2.0
    }

    /// Best estimate of precipitation amount in mm.
    ///
    /// When only a single value is available, returns it directly:
    /// - `(None, Some(max))` — Open-Meteo hourly/daily (only upper bound provided)
    /// - `(Some(min), None)` — hypothetical lower-bound-only provider
    ///
    /// When both bounds are present returns the midpoint.
    /// When neither is present returns 0.
    pub fn amount(&self) -> f32 {
        match (self.amount_min, self.amount_max) {
            (None, Some(max)) => max as f32,
            (Some(min), None) => min as f32,
            _ => self.median(),
        }
    }

    /// Check if this precipitation includes snowfall
    pub fn has_snow(&self) -> bool {
        self.snowfall_cm() > 0.0
    }

    /// Determine if precipitation is primarily snow based on water equivalent ratio
    /// Using Open-Meteo's ratio: 7 cm snow ≈ 10 mm water (0.7 density)
    ///
    /// Note: `snowfall_amount` is stored in tenths of a cm (×10) to preserve one decimal
    /// place of precision. Plain rounding to whole cm would zero out light snow (0.1–0.49 cm).
    ///
    /// Note: uses `amount_max` directly when `amount_min` is absent (e.g. Open-Meteo hourly,
    /// which provides a single precipitation value). Using `median()` in that case would
    /// substitute 0 for the missing min, halving the denominator and making snow detection
    /// twice as permissive as the 60% threshold intends.
    pub fn is_primarily_snow(&self) -> bool {
        let snow_cm = self.snowfall_cm();

        if snow_cm == 0.0 {
            return false;
        }

        let precip_mm = self.amount();

        // Convert snow to water equivalent (7cm snow = 10mm water, so multiply by ~1.43), from open meteo docs
        let snow_water_equivalent = snow_cm * 1.43;

        // If snow water equivalent is more than 60% of total precipitation, it's primarily snow
        snow_water_equivalent > (precip_mm * 0.6)
    }

    /// Get snowfall amount in cm.
    /// `snowfall_amount` is stored as tenths of a cm (×10) for sub-cm precision.
    pub fn snowfall_cm(&self) -> f32 {
        self.snowfall_amount.unwrap_or(0) as f32 / 10.0
    }
}

/// Domain model for astronomical data
/// Sunrise/sunset times are stored as NaiveDateTime (timezone-agnostic wall-clock times)
/// since they represent the actual clock time at the location, not a UTC timestamp
#[derive(Debug, Clone, Copy, Default)]
pub struct Astronomical {
    pub sunrise_time: Option<NaiveDateTime>,
    pub sunset_time: Option<NaiveDateTime>,
}

/// Domain model for hourly weather forecast
/// This is what the application works with, independent of any API
#[derive(Debug, Clone)]
pub struct HourlyForecast {
    pub time: DateTime<Utc>,
    pub temperature: Temperature,
    pub apparent_temperature: Temperature,
    pub wind: Wind,
    pub precipitation: Precipitation,
    pub uv_index: u16,
    pub relative_humidity: u16,
    pub is_night: bool,
    pub cloud_cover: Option<u16>,
    /// Parsed WMO Weather Interpretation Code — `Ok` if recognised, `Err(raw)` if not, `None` if absent
    pub weather_code: Option<Result<WmoWeatherCode, u8>>,
}

/// Domain model for daily weather forecast
/// This is what the application works with, independent of any API
#[derive(Debug, Clone)]
pub struct DailyForecast {
    /// Calendar date (timezone-agnostic) representing the forecast day
    pub date: Option<NaiveDate>,
    pub temp_max: Option<Temperature>,
    pub temp_min: Option<Temperature>,
    pub precipitation: Option<Precipitation>,
    pub astronomical: Option<Astronomical>,
    pub cloud_cover: Option<u16>,
    /// Parsed WMO Weather Interpretation Code — `Ok` if recognised, `Err(raw)` if not, `None` if absent
    pub weather_code: Option<Result<WmoWeatherCode, u8>>,
}

// ============================================================================
// Conversion from BOM models to domain models
// ============================================================================

impl HourlyForecast {
    /// Maps a BOM API hourly entry into the domain model, applying the
    /// configured temperature unit.
    pub(crate) fn from_bom(
        bom: crate::apis::bom::models::HourlyForecast,
        settings: &crate::configs::settings::DashboardSettings,
    ) -> Self {
        let unit = settings.render_options.temp_unit;
        HourlyForecast {
            time: bom.time,
            temperature: Temperature::from(bom.temp).to_unit(unit),
            apparent_temperature: Temperature::from(bom.temp_feels_like).to_unit(unit),
            wind: Wind::new(bom.wind.speed_kilometre, bom.wind.gust_speed_kilometre),
            precipitation: Precipitation::new(
                bom.rain.chance,
                bom.rain.amount.min,
                bom.rain.amount.max,
            ),
            uv_index: bom.uv.unwrap_or_default().0,
            relative_humidity: bom.relative_humidity.0,
            is_night: bom.is_night,
            cloud_cover: None,  // BOM API doesn't provide cloud cover data
            weather_code: None, // BOM API doesn't provide WMO weather codes
        }
    }
}

impl DailyForecast {
    /// Maps a BOM API daily entry into the domain model, applying the
    /// configured temperature unit and display timezone.
    pub(crate) fn from_bom(
        bom: crate::apis::bom::models::DailyEntry,
        settings: &crate::configs::settings::DashboardSettings,
    ) -> Self {
        let unit = settings.render_options.temp_unit;
        let tz = settings.misc.timezone;
        DailyForecast {
            // BOM returns UTC timestamps - convert to local timezone to extract calendar date
            date: bom.date.map(|dt| dt.with_timezone(&tz).date_naive()),
            temp_max: bom.temp_max.map(|t| Temperature::from(t).to_unit(unit)),
            temp_min: bom.temp_min.map(|t| Temperature::from(t).to_unit(unit)),
            precipitation: bom
                .rain
                .map(|r| Precipitation::new(r.chance, r.amount.min, r.amount.max)),
            astronomical: bom.astronomical.map(|a| Astronomical {
                // BOM returns UTC times, convert to local NaiveDateTime for display
                sunrise_time: a.sunrise_time.map(|dt| dt.with_timezone(&tz).naive_local()),
                sunset_time: a.sunset_time.map(|dt| dt.with_timezone(&tz).naive_local()),
            }),
            cloud_cover: None,  // BOM API doesn't provide cloud cover data
            weather_code: None, // BOM API doesn't provide WMO weather codes
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configs::settings::DashboardSettings;

    // Nested modules accumulate here as further conversion/formatting test
    // files are migrated (see docs/test-suite-migration-plan.md Phase 2).
    mod bom_conversion {
        use super::*;
        use crate::apis::bom::models::{DailyForecastResponse, HourlyForecastResponse};
        use std::fs;

        /// Conversion from BOM hourly response to domain models
        #[test]
        fn hourly_to_domain_conversion() {
            let json = fs::read_to_string("tests/fixtures/bom_hourly_forecast.json")
                .expect("failed to read BOM hourly forecast fixture");
            let response: HourlyForecastResponse =
                serde_json::from_str(&json).expect("fixture should deserialize");

            let settings = DashboardSettings::load_test_config().unwrap();
            let domain_forecasts: Vec<HourlyForecast> = response
                .data
                .into_iter()
                .map(|bom| HourlyForecast::from_bom(bom, &settings))
                .collect();

            assert!(!domain_forecasts.is_empty());
            let first = &domain_forecasts[0];
            assert!(first.temperature.value > -50.0 && first.temperature.value < 60.0);
            assert!(first.wind.speed_kmh < 500);
            assert!(first.uv_index < 20);
        }

        /// Conversion from BOM daily response to domain models
        #[test]
        fn daily_to_domain_conversion() {
            let json = fs::read_to_string("tests/fixtures/bom_daily_forecast.json")
                .expect("failed to read BOM daily forecast fixture");
            let response: DailyForecastResponse =
                serde_json::from_str(&json).expect("fixture should deserialize");

            let settings = DashboardSettings::load_test_config().unwrap();
            let domain_forecasts: Vec<DailyForecast> = response
                .data
                .into_iter()
                .map(|bom| DailyForecast::from_bom(bom, &settings))
                .collect();

            assert!(!domain_forecasts.is_empty());
            let first = &domain_forecasts[0];
            if let Some(temp_max) = first.temp_max {
                assert!(temp_max.value > -50.0 && temp_max.value < 60.0);
            }
            if let Some(temp_min) = first.temp_min {
                assert!(temp_min.value > -50.0 && temp_min.value < 60.0);
            }
        }

        /// BOM handles optional precipitation amounts correctly
        #[test]
        fn precipitation_edge_cases() {
            let json = r#"{
                "data": [{
                    "rain": {
                        "amount": {"min": null, "max": null, "units": "mm"},
                        "chance": 0
                    },
                    "temp": 20,
                    "temp_feels_like": 18,
                    "wind": {
                        "speed_knot": 10,
                        "speed_kilometre": 18,
                        "direction": "N",
                        "gust_speed_knot": 15,
                        "gust_speed_kilometre": 28
                    },
                    "relative_humidity": 50,
                    "uv": 5,
                    "time": "2025-10-10T12:00:00Z",
                    "is_night": false
                }]
            }"#;

            let response: HourlyForecastResponse =
                serde_json::from_str(json).expect("inline fixture should deserialize");
            let settings = DashboardSettings::load_test_config().unwrap();
            let domain: Vec<HourlyForecast> = response
                .data
                .into_iter()
                .map(|bom| HourlyForecast::from_bom(bom, &settings))
                .collect();

            let forecast = &domain[0];
            assert_eq!(forecast.precipitation.amount_min, None);
            assert_eq!(forecast.precipitation.amount_max, None);
            assert_eq!(forecast.precipitation.chance, Some(0));
        }

        /// BOM extreme weather values are preserved through conversion
        #[test]
        fn extreme_weather_conversion() {
            let json = r#"{
                "data": [{
                    "rain": {
                        "amount": {"min": 50, "max": 100, "units": "mm"},
                        "chance": 100
                    },
                    "temp": 45,
                    "temp_feels_like": 50,
                    "wind": {
                        "speed_knot": 60,
                        "speed_kilometre": 111,
                        "direction": "S",
                        "gust_speed_knot": 80,
                        "gust_speed_kilometre": 148
                    },
                    "relative_humidity": 95,
                    "uv": 14,
                    "time": "2025-10-10T12:00:00Z",
                    "is_night": false
                }]
            }"#;

            let response: HourlyForecastResponse =
                serde_json::from_str(json).expect("inline fixture should deserialize");
            let settings = DashboardSettings::load_test_config().unwrap();
            let domain: Vec<HourlyForecast> = response
                .data
                .into_iter()
                .map(|bom| HourlyForecast::from_bom(bom, &settings))
                .collect();

            let forecast = &domain[0];
            assert_eq!(forecast.temperature.value, 45.0);
            assert_eq!(forecast.apparent_temperature.value, 50.0);
            assert_eq!(forecast.precipitation.chance, Some(100));
            assert_eq!(forecast.precipitation.amount_min, Some(50));
            assert_eq!(forecast.precipitation.amount_max, Some(100));
            assert_eq!(forecast.wind.speed_kmh, 111);
            assert_eq!(forecast.wind.gust_speed_kmh, 148);
            assert_eq!(forecast.uv_index, 14);
        }

        /// BOM daily forecast with missing optional temperature fields
        #[test]
        fn daily_missing_temps() {
            let json = r#"{
                "data": [{
                    "temp_max": 25,
                    "temp_min": null,
                    "rain": {
                        "amount": {"min": null, "max": null, "units": "mm"},
                        "chance": 10
                    },
                    "astronomical": {
                        "sunrise_time": "2025-10-10T20:00:00Z",
                        "sunset_time": "2025-10-11T09:00:00Z"
                    },
                    "date": "2025-10-10T14:00:00Z"
                }]
            }"#;

            let response: DailyForecastResponse =
                serde_json::from_str(json).expect("inline fixture should deserialize");
            let settings = DashboardSettings::load_test_config().unwrap();
            let domain: Vec<DailyForecast> = response
                .data
                .into_iter()
                .map(|bom| DailyForecast::from_bom(bom, &settings))
                .collect();

            let forecast = &domain[0];
            assert!(forecast.temp_max.is_some());
            assert_eq!(forecast.temp_max.unwrap().value, 25.0);
            assert!(forecast.temp_min.is_none());
        }

        /// BOM converts all hourly forecasts preserving order
        #[test]
        fn hourly_conversion_preserves_order() {
            let json = fs::read_to_string("tests/fixtures/bom_hourly_forecast.json")
                .expect("failed to read BOM hourly forecast fixture");
            let response: HourlyForecastResponse =
                serde_json::from_str(&json).expect("fixture should deserialize");
            let expected_count = response.data.len();

            let settings = DashboardSettings::load_test_config().unwrap();
            let domain_forecasts: Vec<HourlyForecast> = response
                .data
                .into_iter()
                .map(|bom| HourlyForecast::from_bom(bom, &settings))
                .collect();

            assert_eq!(domain_forecasts.len(), expected_count);
            for i in 1..domain_forecasts.len() {
                assert!(
                    domain_forecasts[i].time > domain_forecasts[i - 1].time,
                    "order should be preserved after conversion"
                );
            }
        }
    }

    mod open_meteo_conversion {
        use super::*;
        use crate::apis::open_meteo::models::{OpenMeteoDailyResponse, OpenMeteoHourlyResponse};
        use std::fs;

        #[test]
        fn hourly_conversion() {
            let json = fs::read_to_string("tests/fixtures/open_meteo_hourly_forecast.json")
                .expect("failed to read Open-Meteo hourly forecast fixture");
            let response: OpenMeteoHourlyResponse =
                serde_json::from_str(&json).expect("fixture should deserialize");
            let expected_count = response.hourly.time.len();

            let settings = DashboardSettings::load_test_config().unwrap();
            let domain_forecasts: Vec<HourlyForecast> = response.into_domain(&settings);

            assert_eq!(domain_forecasts.len(), expected_count);
            let first = &domain_forecasts[0];
            assert!(first.temperature.value > -50.0 && first.temperature.value < 60.0);
            assert!(first.wind.speed_kmh < 500);
            assert!(first.uv_index < 20);
        }

        #[test]
        fn daily_conversion() {
            let json = fs::read_to_string("tests/fixtures/open_meteo_daily_forecast.json")
                .expect("failed to read Open-Meteo daily forecast fixture");
            let response: OpenMeteoDailyResponse =
                serde_json::from_str(&json).expect("fixture should deserialize");
            let expected_count = response.daily.time.len();

            let settings = DashboardSettings::load_test_config().unwrap();
            let domain_forecasts: Vec<DailyForecast> = response.into_domain(&settings);

            assert_eq!(domain_forecasts.len(), expected_count);
            let first = &domain_forecasts[0];
            assert!(first.temp_max.is_some());
            assert!(first.temp_min.is_some());
            if let (Some(max), Some(min)) = (first.temp_max, first.temp_min) {
                assert!(max.value >= min.value);
            }
        }

        /// Verifies each hourly forecast picks up the correct value from each
        /// parallel array at its own index, not e.g. every forecast reading index 0.
        #[test]
        fn array_transformation_is_index_correct() {
            let json = r#"{
                "latitude": -37.75, "longitude": 144.875, "timezone": "GMT",
                "timezone_abbreviation": "GMT",
                "current_units": {"time": "iso8601", "interval": "seconds", "is_day": ""},
                "current": {"time": "2025-10-10T12:00", "interval": 900, "is_day": 1},
                "hourly_units": {
                    "time": "iso8601", "temperature_2m": "°C", "apparent_temperature": "°C",
                    "precipitation_probability": "%", "precipitation": "mm", "snowfall": "cm",
                    "uv_index": "", "wind_speed_10m": "km/h", "wind_gusts_10m": "km/h",
                    "relative_humidity_2m": "%"
                },
                "hourly": {
                    "time": ["2025-10-10T00:00", "2025-10-10T01:00"],
                    "temperature_2m": [18.5, 19.2],
                    "apparent_temperature": [15.1, 16.0],
                    "precipitation_probability": [10, 20],
                    "precipitation": [0.0, 0.5], "snowfall": [0.0, 0.0],
                    "uv_index": [0.0, 0.0],
                    "wind_speed_10m": [15.0, 18.0],
                    "wind_gusts_10m": [25.0, 30.0],
                    "relative_humidity_2m": [65, 70],
                    "cloud_cover": [30, 45]
                },
                "daily_units": {
                    "time": "iso8601", "sunrise": "iso8601", "sunset": "iso8601",
                    "temperature_2m_max": "°C", "temperature_2m_min": "°C",
                    "precipitation_sum": "mm", "precipitation_probability_max": "%",
                    "snowfall_sum": "cm"
                },
                "daily": {
                    "time": ["2025-10-10"], "sunrise": ["2025-10-10T06:00"],
                    "sunset": ["2025-10-10T18:00"], "temperature_2m_max": [25.0],
                    "temperature_2m_min": [12.0], "precipitation_sum": [2.5],
                    "precipitation_probability_max": [60], "snowfall_sum": [0.0],
                    "cloud_cover_mean": [55]
                }
            }"#;

            let response: OpenMeteoHourlyResponse =
                serde_json::from_str(json).expect("inline fixture should deserialize");
            let settings = DashboardSettings::load_test_config().unwrap();
            let domain: Vec<HourlyForecast> = response.into_domain(&settings);

            assert_eq!(domain.len(), 2);
            assert_eq!(domain[0].temperature.value, 18.5);
            assert_eq!(domain[0].apparent_temperature.value, 15.1);
            assert_eq!(domain[0].precipitation.chance, Some(10));
            assert_eq!(domain[0].wind.speed_kmh, 15);
            assert_eq!(domain[1].temperature.value, 19.2);
            assert_eq!(domain[1].apparent_temperature.value, 16.0);
            assert_eq!(domain[1].precipitation.chance, Some(20));
            assert_eq!(domain[1].wind.speed_kmh, 18);
        }

        /// `is_night` comes directly from the hourly `is_day` array (each hour
        /// has its own real day/night state from Open-Meteo, not a single
        /// `current` snapshot flag copied onto every hour) — verify the
        /// per-hour mapping.
        #[test]
        fn hourly_is_day_maps_to_is_night_per_hour() {
            let json = r#"{
                "latitude": -37.75, "longitude": 144.875, "timezone": "GMT",
                "current_units": {"time": "iso8601", "interval": "seconds", "is_day": ""},
                "current": {"time": "2025-10-10T12:00", "interval": 900, "is_day": 1},
                "hourly_units": {
                    "time": "iso8601", "temperature_2m": "°C", "apparent_temperature": "°C",
                    "precipitation_probability": "%", "precipitation": "mm", "snowfall": "cm",
                    "uv_index": "", "wind_speed_10m": "km/h", "wind_gusts_10m": "km/h",
                    "relative_humidity_2m": "%"
                },
                "hourly": {
                    "time": ["2025-10-10T00:00", "2025-10-10T06:00", "2025-10-10T12:00"],
                    "temperature_2m": [10.0, 11.0, 20.0],
                    "apparent_temperature": [10.0, 11.0, 20.0],
                    "precipitation_probability": [0, 0, 0],
                    "precipitation": [0.0, 0.0, 0.0], "snowfall": [0.0, 0.0, 0.0],
                    "uv_index": [0.0, 0.0, 5.0],
                    "wind_speed_10m": [5.0, 5.0, 5.0],
                    "wind_gusts_10m": [10.0, 10.0, 10.0],
                    "relative_humidity_2m": [50, 50, 50],
                    "cloud_cover": [0, 0, 0],
                    "is_day": [0, 0, 1]
                },
                "daily_units": {
                    "time": "iso8601", "sunrise": "iso8601", "sunset": "iso8601",
                    "temperature_2m_max": "°C", "temperature_2m_min": "°C",
                    "precipitation_sum": "mm", "precipitation_probability_max": "%",
                    "snowfall_sum": "cm"
                },
                "daily": {
                    "time": ["2025-10-10"], "sunrise": ["2025-10-10T06:21"],
                    "sunset": ["2025-10-10T19:47"], "temperature_2m_max": [25.0],
                    "temperature_2m_min": [12.0], "precipitation_sum": [0.0],
                    "precipitation_probability_max": [0], "snowfall_sum": [0.0]
                }
            }"#;

            let response: OpenMeteoHourlyResponse =
                serde_json::from_str(json).expect("inline fixture should deserialize");
            let settings = DashboardSettings::load_test_config().unwrap();
            let domain: Vec<HourlyForecast> = response.into_domain(&settings);

            assert_eq!(domain.len(), 3);
            // `current.is_day` is 1 (day) for all three hours below, but that's
            // the old snapshot flag this test deliberately ignores -- if
            // `is_night` were still derived from it, every hour would
            // incorrectly report day.
            assert!(domain[0].is_night, "is_day=0 (midnight) => is_night=true");
            assert!(
                domain[1].is_night,
                "is_day=0 (6am, before sunrise) => is_night=true"
            );
            assert!(!domain[2].is_night, "is_day=1 (noon) => is_night=false");
        }

        /// A cache written before `is_day` was requested/parsed won't have it
        /// in the stored JSON; `#[serde(default)]` keeps deserialization from
        /// hard-failing, and the per-index fallback should read as day, not night.
        #[test]
        fn hourly_missing_is_day_defaults_to_day() {
            let json = r#"{
                "latitude": -37.75, "longitude": 144.875, "timezone": "GMT",
                "current_units": {"time": "iso8601", "interval": "seconds", "is_day": ""},
                "current": {"time": "2025-10-10T12:00", "interval": 900, "is_day": 0},
                "hourly_units": {
                    "time": "iso8601", "temperature_2m": "°C", "apparent_temperature": "°C",
                    "precipitation_probability": "%", "precipitation": "mm", "snowfall": "cm",
                    "uv_index": "", "wind_speed_10m": "km/h", "wind_gusts_10m": "km/h",
                    "relative_humidity_2m": "%"
                },
                "hourly": {
                    "time": ["2025-10-10T00:00"],
                    "temperature_2m": [10.0],
                    "apparent_temperature": [10.0],
                    "precipitation_probability": [0],
                    "precipitation": [0.0], "snowfall": [0.0],
                    "uv_index": [0.0],
                    "wind_speed_10m": [5.0],
                    "wind_gusts_10m": [10.0],
                    "relative_humidity_2m": [50],
                    "cloud_cover": [0]
                }
            }"#;

            let response: OpenMeteoHourlyResponse =
                serde_json::from_str(json).expect("inline fixture should deserialize");
            assert!(
                response.hourly.is_day.is_empty(),
                "missing is_day should default to an empty vec, not fail to parse"
            );

            let settings = DashboardSettings::load_test_config().unwrap();
            let domain: Vec<HourlyForecast> = response.into_domain(&settings);

            assert_eq!(domain.len(), 1);
            assert!(
                !domain[0].is_night,
                "a missing is_day entry should default to day, not night"
            );
        }

        #[test]
        fn extreme_values_are_preserved() {
            let json = r#"{
                "latitude": -37.75, "longitude": 144.875, "timezone": "GMT",
                "timezone_abbreviation": "GMT",
                "current_units": {"time": "iso8601", "interval": "seconds", "is_day": ""},
                "current": {"time": "2025-10-10T12:00", "interval": 900, "is_day": 1},
                "hourly_units": {
                    "time": "iso8601", "temperature_2m": "°C", "apparent_temperature": "°C",
                    "precipitation_probability": "%", "precipitation": "mm", "snowfall": "cm",
                    "uv_index": "", "wind_speed_10m": "km/h", "wind_gusts_10m": "km/h",
                    "relative_humidity_2m": "%"
                },
                "hourly": {
                    "time": ["2025-10-10T12:00"],
                    "temperature_2m": [48.5],
                    "apparent_temperature": [55.0],
                    "precipitation_probability": [100],
                    "precipitation": [150.0], "snowfall": [0.0],
                    "uv_index": [15],
                    "wind_speed_10m": [120.0],
                    "wind_gusts_10m": [180.0],
                    "relative_humidity_2m": [99],
                    "cloud_cover": [95]
                },
                "daily_units": {
                    "time": "iso8601", "sunrise": "iso8601", "sunset": "iso8601",
                    "temperature_2m_max": "°C", "temperature_2m_min": "°C",
                    "precipitation_sum": "mm", "precipitation_probability_max": "%",
                    "snowfall_sum": "cm"
                },
                "daily": {
                    "time": ["2025-10-10"], "sunrise": ["2025-10-10T06:00"],
                    "sunset": ["2025-10-10T18:00"], "temperature_2m_max": [50.0],
                    "temperature_2m_min": [-10.0], "precipitation_sum": [200.0],
                    "precipitation_probability_max": [100], "snowfall_sum": [0.0],
                    "cloud_cover_mean": [98]
                }
            }"#;

            let response: OpenMeteoHourlyResponse =
                serde_json::from_str(json).expect("inline fixture should deserialize");
            let settings = DashboardSettings::load_test_config().unwrap();
            let domain: Vec<HourlyForecast> = response.into_domain(&settings);

            let forecast = &domain[0];
            assert_eq!(forecast.temperature.value, 48.5);
            assert_eq!(forecast.apparent_temperature.value, 55.0);
            assert_eq!(forecast.precipitation.chance, Some(100));
            assert_eq!(forecast.wind.speed_kmh, 120);
            assert_eq!(forecast.wind.gust_speed_kmh, 180);
            assert_eq!(forecast.uv_index, 15);
        }

        #[test]
        fn daily_conversion_preserves_temp_min_max_relationship() {
            let json = fs::read_to_string("tests/fixtures/open_meteo_daily_forecast.json")
                .expect("failed to read Open-Meteo daily forecast fixture");
            let response: OpenMeteoDailyResponse =
                serde_json::from_str(&json).expect("fixture should deserialize");
            let settings = DashboardSettings::load_test_config().unwrap();
            let domain_forecasts: Vec<DailyForecast> = response.into_domain(&settings);

            for forecast in &domain_forecasts {
                if let (Some(max), Some(min)) = (forecast.temp_max, forecast.temp_min) {
                    assert!(max.value >= min.value);
                }
            }
        }

        #[test]
        fn zero_precipitation_is_handled() {
            let json = r#"{
                "latitude": -37.75, "longitude": 144.875, "timezone": "GMT",
                "timezone_abbreviation": "GMT",
                "current_units": {"time": "iso8601", "interval": "seconds", "is_day": ""},
                "current": {"time": "2025-10-10T12:00", "interval": 900, "is_day": 1},
                "hourly_units": {
                    "time": "iso8601", "temperature_2m": "°C", "apparent_temperature": "°C",
                    "precipitation_probability": "%", "precipitation": "mm", "snowfall": "cm",
                    "uv_index": "", "wind_speed_10m": "km/h", "wind_gusts_10m": "km/h",
                    "relative_humidity_2m": "%"
                },
                "hourly": {
                    "time": ["2025-10-10T12:00"],
                    "temperature_2m": [20.0],
                    "apparent_temperature": [18.0],
                    "precipitation_probability": [0],
                    "precipitation": [0.0], "snowfall": [0.0],
                    "uv_index": [5.0],
                    "wind_speed_10m": [10.0],
                    "wind_gusts_10m": [15.0],
                    "relative_humidity_2m": [50],
                    "cloud_cover": [10]
                },
                "daily_units": {
                    "time": "iso8601", "sunrise": "iso8601", "sunset": "iso8601",
                    "temperature_2m_max": "°C", "temperature_2m_min": "°C",
                    "precipitation_sum": "mm", "precipitation_probability_max": "%",
                    "snowfall_sum": "cm"
                },
                "daily": {
                    "time": ["2025-10-10"], "sunrise": ["2025-10-10T06:00"],
                    "sunset": ["2025-10-10T18:00"], "temperature_2m_max": [25.0],
                    "temperature_2m_min": [15.0], "precipitation_sum": [0.0],
                    "precipitation_probability_max": [0], "snowfall_sum": [0.0],
                    "cloud_cover_mean": [5]
                }
            }"#;

            let response: OpenMeteoHourlyResponse =
                serde_json::from_str(json).expect("inline fixture should deserialize");
            let settings = DashboardSettings::load_test_config().unwrap();
            let domain: Vec<HourlyForecast> = response.into_domain(&settings);

            // Open-Meteo doesn't provide min/max per hour, just total
            assert_eq!(domain[0].precipitation.chance, Some(0));
        }

        #[test]
        fn conversion_preserves_chronological_order() {
            let json = fs::read_to_string("tests/fixtures/open_meteo_hourly_forecast.json")
                .expect("failed to read Open-Meteo hourly forecast fixture");
            let response: OpenMeteoHourlyResponse =
                serde_json::from_str(&json).expect("fixture should deserialize");
            let settings = DashboardSettings::load_test_config().unwrap();
            let domain_forecasts: Vec<HourlyForecast> = response.into_domain(&settings);

            for i in 1..domain_forecasts.len() {
                assert!(domain_forecasts[i].time > domain_forecasts[i - 1].time);
            }
        }

        #[test]
        fn hourly_conversion_preserves_snowfall() {
            let json = fs::read_to_string("tests/fixtures/open_meteo_hourly_forecast.json")
                .expect("failed to read Open-Meteo hourly forecast fixture");
            let response: OpenMeteoHourlyResponse =
                serde_json::from_str(&json).expect("fixture should deserialize");
            let settings = DashboardSettings::load_test_config().unwrap();
            let hourly_forecasts: Vec<HourlyForecast> = response.into_domain(&settings);

            let with_snow = hourly_forecasts.iter().find(|f| f.precipitation.has_snow());
            assert!(
                with_snow.is_some(),
                "expected at least one forecast with snowfall in test fixture"
            );
            let snowfall = with_snow.unwrap().precipitation.snowfall_amount.unwrap();
            assert!(snowfall > 0);
        }

        #[test]
        fn daily_conversion_preserves_snowfall_sum() {
            let json = fs::read_to_string("tests/fixtures/open_meteo_daily_forecast.json")
                .expect("failed to read Open-Meteo daily forecast fixture");
            let response: OpenMeteoDailyResponse =
                serde_json::from_str(&json).expect("fixture should deserialize");
            let settings = DashboardSettings::load_test_config().unwrap();
            let daily_forecasts: Vec<DailyForecast> = response.into_domain(&settings);

            let with_snow = daily_forecasts.iter().find(|f| {
                f.precipitation
                    .as_ref()
                    .map(|p| p.has_snow())
                    .unwrap_or(false)
            });
            assert!(
                with_snow.is_some(),
                "expected at least one daily forecast with snowfall in test fixture"
            );
        }

        #[test]
        fn zero_snowfall_is_handled_correctly() {
            let json = r#"{
                "latitude": 45.38, "longitude": -81.109, "timezone": "America/Toronto",
                "timezone_abbreviation": "EST",
                "current_units": {"time": "iso8601", "interval": "seconds", "is_day": ""},
                "current": {"time": "2026-01-18T12:00", "interval": 900, "is_day": 1},
                "hourly_units": {
                    "time": "iso8601", "temperature_2m": "°C", "apparent_temperature": "°C",
                    "precipitation_probability": "%", "precipitation": "mm", "snowfall": "cm",
                    "wind_speed_10m": "km/h", "wind_gusts_10m": "km/h", "uv_index": "",
                    "relative_humidity_2m": "%", "cloud_cover": "%", "is_day": ""
                },
                "hourly": {
                    "time": ["2026-01-18T12:00"],
                    "temperature_2m": [5.0],
                    "apparent_temperature": [3.0],
                    "precipitation_probability": [20],
                    "precipitation": [1.0],
                    "snowfall": [0.0],
                    "wind_speed_10m": [10],
                    "wind_gusts_10m": [15],
                    "uv_index": [2],
                    "relative_humidity_2m": [70],
                    "cloud_cover": [40],
                    "is_day": [1]
                }
            }"#;

            let response: OpenMeteoHourlyResponse =
                serde_json::from_str(json).expect("inline fixture should deserialize");
            let settings = DashboardSettings::load_test_config().unwrap();
            let domain: Vec<HourlyForecast> = response.into_domain(&settings);

            assert_eq!(domain.len(), 1);
            assert_eq!(domain[0].precipitation.snowfall_amount, Some(0));
            assert!(!domain[0].precipitation.has_snow());
            assert!(!domain[0].precipitation.is_primarily_snow());
        }
    }

    /// Australia (Melbourne/Sydney) DST: starts first Sunday in October,
    /// 2:00 AM -> 3:00 AM (AEST -> AEDT); ends first Sunday in April,
    /// 3:00 AM -> 2:00 AM (AEDT -> AEST). Verifies API responses convert to
    /// domain models with correct local time across both transitions.
    mod daylight_saving {
        use super::*;
        use crate::apis::open_meteo::models::OpenMeteoHourlyResponse;
        use chrono::{Datelike, Timelike};

        /// BOM returns UTC times; verify the 1-hour gap at spring forward
        /// (2 AM doesn't exist) on Oct 5, 2025.
        #[test]
        fn bom_forecast_time_conversion_during_dst() {
            use crate::apis::bom::models::HourlyForecast as BomHourlyForecast;

            let settings = DashboardSettings::load_test_config().unwrap();

            let test_cases = vec![
                ("2025-10-04T15:00:00Z", 1, 5, "before DST"),
                ("2025-10-04T16:00:00Z", 3, 5, "after DST - skipped 2 AM"),
                ("2025-10-04T17:00:00Z", 4, 5, "after DST"),
            ];

            for (utc_time, expected_hour, expected_day, description) in test_cases {
                let json = format!(
                    r#"{{
                        "rain": {{"amount": {{"min": null, "max": null, "units": "mm"}}, "chance": 10}},
                        "temp": 18,
                        "temp_feels_like": 16,
                        "wind": {{
                            "speed_knot": 8, "speed_kilometre": 15, "direction": "N",
                            "gust_speed_knot": 12, "gust_speed_kilometre": 22
                        }},
                        "relative_humidity": 65,
                        "uv": 5,
                        "time": "{utc_time}",
                        "is_night": false
                    }}"#
                );

                let bom_forecast: BomHourlyForecast =
                    serde_json::from_str(&json).expect("inline fixture should deserialize");
                let domain_forecast = HourlyForecast::from_bom(bom_forecast, &settings);
                let local_time = domain_forecast.time.with_timezone(&settings.misc.timezone);

                assert_eq!(local_time.hour(), expected_hour, "{description}");
                assert_eq!(local_time.day(), expected_day, "{description}");
            }
        }

        /// Open-Meteo returns UTC times (timezone=UTC); verify the duplicate
        /// local hour at fall back (2 AM happens twice) on Apr 6, 2025.
        #[test]
        fn open_meteo_forecast_time_conversion_during_dst() {
            let json = r#"{
                "latitude": -37.75, "longitude": 144.875, "timezone": "UTC",
                "timezone_abbreviation": "UTC",
                "current_units": {"time": "iso8601", "interval": "seconds", "is_day": ""},
                "current": {"time": "2025-04-05T14:00", "interval": 900, "is_day": 1},
                "hourly_units": {
                    "time": "iso8601", "temperature_2m": "°C", "apparent_temperature": "°C",
                    "precipitation_probability": "%", "precipitation": "mm", "snowfall": "cm",
                    "uv_index": "", "wind_speed_10m": "km/h", "wind_gusts_10m": "km/h",
                    "relative_humidity_2m": "%"
                },
                "hourly": {
                    "time": [
                        "2025-04-05T14:00", "2025-04-05T15:00",
                        "2025-04-05T16:00", "2025-04-05T17:00"
                    ],
                    "temperature_2m": [19.5, 18.5, 17.8, 17.0],
                    "apparent_temperature": [18.2, 17.2, 16.5, 15.8],
                    "precipitation_probability": [10, 20, 15, 10],
                    "precipitation": [0.0, 0.0, 0.0, 0.0],
                    "snowfall": [0.0, 0.0, 0.0, 0.0],
                    "uv_index": [1, 2, 3, 4],
                    "wind_speed_10m": [10, 15, 12, 10],
                    "wind_gusts_10m": [18, 22, 18, 15],
                    "relative_humidity_2m": [80, 75, 78, 80],
                    "cloud_cover": [40, 35, 28, 25]
                },
                "daily_units": {
                    "time": "iso8601", "sunrise": "iso8601", "sunset": "iso8601",
                    "temperature_2m_max": "°C", "temperature_2m_min": "°C",
                    "precipitation_sum": "mm", "precipitation_probability_max": "%",
                    "snowfall_sum": "cm"
                },
                "daily": {
                    "time": ["2025-04-06"], "sunrise": ["2025-04-06T07:15"],
                    "sunset": ["2025-04-06T18:30"], "temperature_2m_max": [22.5],
                    "temperature_2m_min": [15.2], "precipitation_sum": [0.0],
                    "precipitation_probability_max": [20], "snowfall_sum": [0.0],
                    "cloud_cover_mean": [32]
                }
            }"#;

            let response: OpenMeteoHourlyResponse =
                serde_json::from_str(json).expect("inline fixture should deserialize");
            let settings = DashboardSettings::load_test_config().unwrap();
            let domain_forecasts: Vec<HourlyForecast> = response.into_domain(&settings);

            assert_eq!(domain_forecasts.len(), 4);

            let test_cases = vec![
                (0, 1, 6, "before fall back: 1 AM AEDT"),
                (1, 2, 6, "before fall back: 2 AM AEDT"),
                (2, 2, 6, "after fall back: 2 AM AEST (duplicate hour!)"),
                (3, 3, 6, "after fall back: 3 AM AEST"),
            ];

            for (index, expected_hour, expected_day, description) in test_cases {
                let forecast = &domain_forecasts[index];
                let local_time = forecast.time.with_timezone(&settings.misc.timezone);
                assert_eq!(local_time.hour(), expected_hour, "{description}");
                assert_eq!(local_time.day(), expected_day, "{description}");
            }

            // Sequence should be 1, 2, 2, 3 — 2 AM happens twice during fall back.
            let local_hours: Vec<u32> = domain_forecasts
                .iter()
                .map(|f| f.time.with_timezone(&settings.misc.timezone).hour())
                .collect();
            assert_eq!(local_hours, vec![1, 2, 2, 3]);
        }
    }

    mod wind_speed_conversion {
        use super::*;
        use crate::configs::settings::WindSpeedUnit;

        #[test]
        fn kmh_no_conversion() {
            assert_eq!(Wind::convert_speed(20, WindSpeedUnit::KmH), 20);
        }

        #[test]
        fn kmh_to_mph() {
            // 20 km/h * 0.621371 = 12.42742 ≈ 12 mph
            assert_eq!(Wind::convert_speed(20, WindSpeedUnit::Mph), 12);
        }

        #[test]
        fn kmh_to_knots() {
            // 20 km/h * 0.539957 = 10.79914 ≈ 11 knots
            assert_eq!(Wind::convert_speed(20, WindSpeedUnit::Knots), 11);
        }

        #[test]
        fn zero_speed() {
            assert_eq!(Wind::convert_speed(0, WindSpeedUnit::KmH), 0);
            assert_eq!(Wind::convert_speed(0, WindSpeedUnit::Mph), 0);
            assert_eq!(Wind::convert_speed(0, WindSpeedUnit::Knots), 0);
        }

        #[test]
        fn high_values() {
            assert_eq!(Wind::convert_speed(100, WindSpeedUnit::KmH), 100);
            assert_eq!(Wind::convert_speed(100, WindSpeedUnit::Mph), 62); // 100 * 0.621371 ≈ 62
            assert_eq!(Wind::convert_speed(100, WindSpeedUnit::Knots), 54); // 100 * 0.539957 ≈ 54
        }

        #[test]
        fn speed_in_unit_without_gust() {
            let wind = Wind::new(20, 30);
            assert_eq!(wind.speed_in_unit(false, WindSpeedUnit::KmH), 20);
            assert_eq!(wind.speed_in_unit(false, WindSpeedUnit::Mph), 12);
            assert_eq!(wind.speed_in_unit(false, WindSpeedUnit::Knots), 11);
        }

        #[test]
        fn speed_in_unit_with_gust() {
            let wind = Wind::new(20, 30);
            assert_eq!(wind.speed_in_unit(true, WindSpeedUnit::KmH), 30);
            assert_eq!(wind.speed_in_unit(true, WindSpeedUnit::Mph), 19); // 30 * 0.621371 ≈ 19
            assert_eq!(wind.speed_in_unit(true, WindSpeedUnit::Knots), 16); // 30 * 0.539957 ≈ 16
        }

        #[test]
        fn conversion_factors_are_accurate() {
            // 1 km/h = 0.621371 mph, 1 km/h = 0.539957 knots
            let test_cases = vec![(10, 6, 5), (50, 31, 27), (80, 50, 43), (120, 75, 65)];
            for (kmh, expected_mph, expected_knots) in test_cases {
                assert_eq!(
                    Wind::convert_speed(kmh, WindSpeedUnit::Mph),
                    expected_mph,
                    "failed for {kmh} km/h to mph"
                );
                assert_eq!(
                    Wind::convert_speed(kmh, WindSpeedUnit::Knots),
                    expected_knots,
                    "failed for {kmh} km/h to knots"
                );
            }
        }
    }

    /// `is_primarily_snow()` determines whether precipitation is primarily
    /// snow based on the snowfall amount and total precipitation, using the
    /// 1.43 conversion factor (7cm snow = 10mm water). The 60% threshold:
    /// >= 60% snow water equivalent -> snow icon, < 60% -> rain icon.
    mod snow_detection {
        use super::*;

        #[test]
        fn high_snow_ratio_is_primarily_snow() {
            // 10cm snow × 1.43 = 14.3mm water, total precip = 8mm -> 178.75%
            let precip = Precipitation::new_with_snowfall(Some(80), Some(7), Some(9), Some(100));
            assert!(precip.is_primarily_snow());
        }

        #[test]
        fn boundary_just_above_60_percent_is_snow() {
            // Open-Meteo ratio: 7cm snow = 10mm water. Median precip = 10mm.
            // 4.3cm -> 6.14mm > 6.0mm (60% of 10mm) -> snow.
            let precip = Precipitation::new_with_snowfall(Some(75), Some(9), Some(11), Some(43));
            assert!(precip.is_primarily_snow());
        }

        #[test]
        fn boundary_just_below_60_percent_is_not_snow() {
            // 4.1cm -> 5.86mm < 6.0mm -> not snow.
            let precip = Precipitation::new_with_snowfall(Some(75), Some(9), Some(11), Some(41));
            assert!(!precip.is_primarily_snow());
        }

        #[test]
        fn below_threshold_is_not_snow() {
            // 3cm snow = ~4.29mm water, total = 10mm -> 42.9% snow
            let precip = Precipitation::new_with_snowfall(Some(70), Some(9), Some(11), Some(30));
            assert!(!precip.is_primarily_snow());
        }

        #[test]
        fn just_above_threshold_is_snow() {
            // 4.5cm snow → 6.435mm water, total = 10mm → 64.35%
            let precip = Precipitation::new_with_snowfall(Some(75), Some(9), Some(11), Some(45));
            assert!(precip.is_primarily_snow());
        }

        #[test]
        fn just_below_threshold_is_not_snow() {
            // 4cm snow = ~5.72mm water, total = 10mm -> 57.2% snow
            let precip = Precipitation::new_with_snowfall(Some(75), Some(9), Some(11), Some(40));
            assert!(!precip.is_primarily_snow());
        }

        #[test]
        fn no_snowfall_field_returns_false() {
            let precip = Precipitation::new(Some(80), Some(10), Some(20));
            assert!(!precip.is_primarily_snow());
        }

        #[test]
        fn zero_snowfall_returns_false() {
            let precip = Precipitation::new_with_snowfall(Some(60), Some(5), Some(10), Some(0));
            assert!(!precip.is_primarily_snow());
        }

        #[test]
        fn all_snow_no_rain() {
            // 14.3cm snow × 1.43 = 20.4mm water, total = 10mm -> 204%
            let precip = Precipitation::new_with_snowfall(Some(90), Some(9), Some(11), Some(143));
            assert!(precip.is_primarily_snow());
        }

        #[test]
        fn light_snow_with_heavy_rain_is_not_primarily_snow() {
            // 1cm snow = ~1.43mm water, total = 20mm -> 7.15% snow
            let precip = Precipitation::new_with_snowfall(Some(85), Some(18), Some(22), Some(10));
            assert!(!precip.is_primarily_snow());
        }

        #[test]
        fn winter_storm_scenario_is_snow() {
            // 20cm snow × 1.43 = 28.6mm water, total = 15mm -> 190%
            let precip = Precipitation::new_with_snowfall(Some(95), Some(14), Some(16), Some(200));
            assert!(precip.is_primarily_snow());
        }

        #[test]
        fn mixed_precipitation_scenario_is_not_snow() {
            // 3cm snow = ~4.29mm water, total = 12mm -> 35.75% snow
            let precip = Precipitation::new_with_snowfall(Some(80), Some(11), Some(13), Some(30));
            assert!(!precip.is_primarily_snow());
        }

        #[test]
        fn light_flurries_scenario_is_snow() {
            // 1cm snow = ~1.43mm water, total = 1mm -> 143% snow (all snow)
            let precip = Precipitation::new_with_snowfall(Some(40), Some(0), Some(1), Some(10));
            assert!(precip.is_primarily_snow());
        }

        #[test]
        fn has_snow_true_with_snowfall() {
            let precip = Precipitation::new_with_snowfall(Some(50), Some(5), Some(10), Some(10));
            assert!(precip.has_snow());
        }

        #[test]
        fn has_snow_false_without_snowfall() {
            let precip = Precipitation::new(Some(50), Some(5), Some(10));
            assert!(!precip.has_snow());
        }

        #[test]
        fn has_snow_false_with_zero_snowfall() {
            let precip = Precipitation::new_with_snowfall(Some(50), Some(5), Some(10), Some(0));
            assert!(!precip.has_snow());
        }
    }
}
