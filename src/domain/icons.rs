use super::models::{DailyForecast, HourlyForecast, Precipitation, Wind};
use crate::logger;
use crate::weather::icons::{
    DayNight, Icon, PrecipitationChanceName, PrecipitationKind, RainAmountIcon, WindIconName,
};
use crate::weather::utils::moon_phase_icon_name;
use crate::CONFIG;

// ============================================================================
// Icon implementations for domain models
// ============================================================================

impl Icon for Wind {
    fn icon_name(&self) -> String {
        let speed = self.speed(CONFIG.render_options.use_gust_instead_of_wind);
        match speed {
            0..=20 => WindIconName::Wind,
            21..=40 => WindIconName::UmbrellaWind,
            41.. => WindIconName::UmbrellaWindAlt,
        }
        .to_string()
    }
}

/// If `use_moon_phase_instead_of_clear_night` is enabled and the icon represents
/// a clear night, replaces it with the current moon phase icon.
fn apply_moon_phase_override(icon: String, is_night: bool) -> String {
    if !is_night || !CONFIG.render_options.use_moon_phase_instead_of_clear_night {
        return icon;
    }

    let clear_night_suffix = format!("{}{}.svg", PrecipitationChanceName::Clear, DayNight::Night);

    if icon.ends_with(&clear_night_suffix) {
        logger::detail("Using moon phase icon instead of clear night");
        moon_phase_icon_name().to_string()
    } else {
        icon
    }
}

/// Converts a percentage (0–100) to a `PrecipitationChanceName`.
/// Used for both cloud cover data and precipitation chance fallback.
#[must_use]
fn percentage_to_cloud_name(pct: u16) -> PrecipitationChanceName {
    match pct {
        0..=25 => PrecipitationChanceName::Clear,
        26..=50 => PrecipitationChanceName::PartlyCloudy,
        51..=75 => PrecipitationChanceName::Overcast,
        76.. => PrecipitationChanceName::Extreme,
    }
}

/// Converts the precipitation amount to a corresponding `PrecipitationKind`.
///
/// `is_hourly`: if true, scales the amount to a daily equivalent before classifying.
#[must_use]
fn precipitation_amount_to_name(precip: &Precipitation, is_hourly: bool) -> PrecipitationKind {
    let mut median = precip.median();

    if is_hourly {
        median *= 24.0;
    }

    // If primarily snow, return snow variant instead of rain
    if precip.is_primarily_snow() {
        return match median {
            0.0..1.4 => PrecipitationKind::None,
            _ => PrecipitationKind::Snow,
        };
    }

    match median {
        0.0..3.0 => PrecipitationKind::None,
        3.0..=20.0 => PrecipitationKind::Drizzle,
        _ => PrecipitationKind::Rain,
    }
}

/// Ensures precipitation amount requires minimum cloud coverage.
/// Heavy precipitation cannot occur with completely clear skies.
///
/// # Arguments
///
/// * `cloud_name` - Cloud cover level from cloud data or precipitation chance
/// * `amount_name` - Precipitation amount (None, Drizzle, or Rain)
///
/// # Returns
///
/// * Adjusted cloud level ensuring consistency with precipitation amount
#[must_use]
fn apply_precipitation_override(
    cloud_name: PrecipitationChanceName,
    amount_name: PrecipitationKind,
) -> PrecipitationChanceName {
    match amount_name {
        PrecipitationKind::None => cloud_name,
        PrecipitationKind::Drizzle => {
            // Drizzle requires at least partly cloudy
            match cloud_name {
                PrecipitationChanceName::Clear => PrecipitationChanceName::PartlyCloudy,
                _ => cloud_name,
            }
        }
        PrecipitationKind::Rain => {
            // Heavy rain requires at least overcast
            match cloud_name {
                PrecipitationChanceName::Clear | PrecipitationChanceName::PartlyCloudy => {
                    PrecipitationChanceName::Overcast
                }
                _ => cloud_name,
            }
        }
        PrecipitationKind::Snow => {
            // Snow requires at least partly cloudy
            match cloud_name {
                PrecipitationChanceName::Clear => PrecipitationChanceName::PartlyCloudy,
                _ => cloud_name,
            }
        }
        PrecipitationKind::Sleet => {
            // Sleet (freezing rain/drizzle) requires at least partly cloudy
            match cloud_name {
                PrecipitationChanceName::Clear => PrecipitationChanceName::PartlyCloudy,
                _ => cloud_name,
            }
        }
        PrecipitationKind::Hail => {
            // Hail requires at least overcast (severe weather)
            match cloud_name {
                PrecipitationChanceName::Clear | PrecipitationChanceName::PartlyCloudy => {
                    PrecipitationChanceName::Overcast
                }
                _ => cloud_name,
            }
        }
        PrecipitationKind::Fog => {
            // Fog can occur with any cloud cover, no override needed
            cloud_name
        }
    }
}

impl Icon for Precipitation {
    fn icon_name(&self) -> String {
        RainAmountIcon::RainAmount.to_string()
    }
}

impl Icon for DailyForecast {
    fn icon_name(&self) -> String {
        // Priority 1: Use WMO weather code if available (most accurate)
        if CONFIG.render_options.prefer_weather_codes {
            if let Some(code) = self.weather_code {
                logger::debug("DailyForecast: Using WMO weather code for icon selection");
                let wmo_code = crate::domain::weather_code::WmoWeatherCode::from(code);
                // Daily forecasts always use day icons
                return wmo_code.to_icon_name(false);
            }
            logger::debug("DailyForecast: WMO weather code not available, falling back to precipitation-based icon logic");
        }

        logger::debug("DailyForecast: Falling back to precipitation-based icon logic");

        // Priority 2: Fall back to precipitation-based logic (BOM provider, missing codes)
        if let Some(ref precip) = self.precipitation {
            // Determine cloud coverage from cloud_cover data if available, otherwise fall back to precipitation chance
            let raw_cloud_name = if let Some(cloud_cover) = self.cloud_cover {
                percentage_to_cloud_name(cloud_cover)
            } else {
                percentage_to_cloud_name(precip.chance.unwrap_or(0))
            };

            let amount_name = precipitation_amount_to_name(precip, false);

            // Apply precipitation override: ensure heavy rain requires adequate cloud cover
            // Note: After override, Clear can only occur with amount_name = None
            let cloud_name = apply_precipitation_override(raw_cloud_name, amount_name);

            format!("{cloud_name}{}{amount_name}.svg", DayNight::Day)
        } else {
            // Default to clear day if no precipitation data
            format!("{}{}.svg", PrecipitationChanceName::Clear, DayNight::Day)
        }
    }
}

impl Icon for HourlyForecast {
    fn icon_name(&self) -> String {
        // Priority 1: Use WMO weather code if available (most accurate)
        if CONFIG.render_options.prefer_weather_codes {
            if let Some(code) = self.weather_code {
                logger::debug("HourlyForecast: Using WMO weather code for icon selection");
                let wmo_code = crate::domain::weather_code::WmoWeatherCode::from(code);
                let icon = wmo_code.to_icon_name(self.is_night);
                return apply_moon_phase_override(icon, self.is_night);
            }
            logger::debug("HourlyForecast: WMO weather code not available, falling back to precipitation-based icon logic");
        }

        logger::debug("HourlyForecast: Falling back to precipitation-based icon logic");

        // Priority 2: Fall back to cloud_cover + precipitation logic (BOM provider, missing codes)
        // Determine cloud coverage from cloud_cover data if available, otherwise fall back to precipitation chance
        let raw_cloud_name = if let Some(cloud_cover) = self.cloud_cover {
            percentage_to_cloud_name(cloud_cover)
        } else {
            percentage_to_cloud_name(self.precipitation.chance.unwrap_or(0))
        };

        let amount_name = precipitation_amount_to_name(&self.precipitation, true);
        let day_night = if self.is_night {
            DayNight::Night
        } else {
            DayNight::Day
        };

        // Apply precipitation override: ensure heavy rain requires adequate cloud cover
        // Note: After override, Clear can only occur with amount_name = None
        let cloud_name = apply_precipitation_override(raw_cloud_name, amount_name);

        let icon = format!("{cloud_name}{day_night}{amount_name}.svg");
        apply_moon_phase_override(icon, self.is_night)
    }
}
