use chrono::{DateTime, Utc};
use std::{
    fmt::{self, Display},
    ops::Deref,
};

use crate::configs::settings::TemperatureUnit;

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

    pub fn get_speed(&self, use_gust: bool) -> u16 {
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
    pub fn get_speed_in_unit(
        &self,
        use_gust: bool,
        unit: crate::configs::settings::WindSpeedUnit,
    ) -> u16 {
        let speed_kmh = self.get_speed(use_gust);
        Self::convert_speed(speed_kmh, unit)
    }
}

/// Domain model for precipitation information
#[derive(Debug, Clone)]
pub struct Precipitation {
    pub chance: Option<u16>,
    pub amount_min: Option<u16>,
    pub amount_max: Option<u16>,
}

impl Precipitation {
    pub fn new(chance: Option<u16>, amount_min: Option<u16>, amount_max: Option<u16>) -> Self {
        Self {
            chance,
            amount_min,
            amount_max,
        }
    }

    pub fn calculate_median(&self) -> f32 {
        let min = self.amount_min.unwrap_or(0);
        let max = self.amount_max.unwrap_or(min);
        (min + max) as f32 / 2.0
    }
}

/// Domain model for astronomical data
#[derive(Debug, Clone, Copy, Default)]
pub struct Astronomical {
    pub sunrise_time: Option<DateTime<Utc>>,
    pub sunset_time: Option<DateTime<Utc>>,
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
}

/// Domain model for daily weather forecast
/// This is what the application works with, independent of any API
#[derive(Debug, Clone)]
pub struct DailyForecast {
    pub date: Option<DateTime<Utc>>,
    pub temp_max: Option<Temperature>,
    pub temp_min: Option<Temperature>,
    pub precipitation: Option<Precipitation>,
    pub astronomical: Option<Astronomical>,
    pub cloud_cover: Option<u16>,
}

// ============================================================================
// Conversion from BOM models to domain models
// ============================================================================

impl From<crate::apis::bom::models::HourlyForecast> for HourlyForecast {
    fn from(bom: crate::apis::bom::models::HourlyForecast) -> Self {
        HourlyForecast {
            time: bom.time,
            temperature: bom.temp.into(),
            apparent_temperature: bom.temp_feels_like.into(),
            wind: Wind::new(bom.wind.speed_kilometre, bom.wind.gust_speed_kilometre),
            precipitation: Precipitation::new(
                bom.rain.chance,
                bom.rain.amount.min,
                bom.rain.amount.max,
            ),
            uv_index: bom.uv.unwrap_or_default().0,
            relative_humidity: bom.relative_humidity.0,
            is_night: bom.is_night,
            cloud_cover: None, // BOM API doesn't provide cloud cover data
        }
    }
}

impl From<crate::apis::bom::models::DailyEntry> for DailyForecast {
    fn from(bom: crate::apis::bom::models::DailyEntry) -> Self {
        DailyForecast {
            date: bom.date,
            temp_max: bom.temp_max.map(|t| t.into()),
            temp_min: bom.temp_min.map(|t| t.into()),
            precipitation: bom
                .rain
                .map(|r| Precipitation::new(r.chance, r.amount.min, r.amount.max)),
            astronomical: bom.astronomical.map(|a| Astronomical {
                sunrise_time: a.sunrise_time,
                sunset_time: a.sunset_time,
            }),
            cloud_cover: None, // BOM API doesn't provide cloud cover data
        }
    }
}
