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
    pub fn from_bom(
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
    pub fn from_bom(
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
