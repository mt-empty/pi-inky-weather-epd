use super::models::{DailyForecast, HourlyForecast, Precipitation, Wind};
use crate::logger;
use crate::weather::icons::{
    DayNight, HumidityIconName, Icon, RainAmountIcon, RainAmountName, RainChanceName, UVIndexIcon,
    WindIconName,
};
use crate::weather::utils::get_moon_phase_icon_name;
use crate::CONFIG;

// ============================================================================
// Icon implementations for domain models
// ============================================================================

impl Icon for Wind {
    fn get_icon_name(&self) -> String {
        let speed = self.get_speed(CONFIG.render_options.use_gust_instead_of_wind);
        match speed {
            0..=20 => WindIconName::Wind,
            21..=40 => WindIconName::UmbrellaWind,
            41.. => WindIconName::UmbrellaWindAlt,
        }
        .to_string()
    }
}

impl Precipitation {
    /// Converts the precipitation amount to a corresponding `RainAmountName`.
    ///
    /// # Arguments
    ///
    /// * `is_hourly` - If true, treats the precipitation amount as hourly and scales accordingly.
    ///
    /// # Returns
    ///
    /// * A `RainAmountName` variant representing the precipitation amount.
    pub fn amount_to_name(&self, is_hourly: bool) -> RainAmountName {
        let mut median = self.calculate_median();

        if is_hourly {
            median *= 24.0;
        }
        match median {
            0.0..=2.0 => RainAmountName::None,
            3.0..=20.0 => RainAmountName::Drizzle,
            21.0.. => RainAmountName::Rain,
            _ => RainAmountName::None,
        }
    }

    /// Converts the precipitation chance (percentage) to a corresponding `RainChanceName`.
    ///
    /// # Returns
    ///
    /// * A `RainChanceName` variant representing the precipitation chance.
    pub fn chance_to_name(&self) -> RainChanceName {
        match self.chance.unwrap_or(0) {
            0..=25 => RainChanceName::Clear,
            26..=50 => RainChanceName::PartlyCloudy,
            51..=75 => RainChanceName::Overcast,
            76.. => RainChanceName::Extreme,
        }
    }
}

/// Converts cloud cover percentage to a corresponding `RainChanceName`.
///
/// # Arguments
///
/// * `cloud_cover` - Cloud cover percentage (0-100)
///
/// # Returns
///
/// * A `RainChanceName` variant representing the cloud cover level
fn cloud_cover_to_name(cloud_cover: u16) -> RainChanceName {
    match cloud_cover {
        0..=25 => RainChanceName::Clear,
        26..=50 => RainChanceName::PartlyCloudy,
        51..=75 => RainChanceName::Overcast,
        76.. => RainChanceName::Extreme,
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
fn apply_precipitation_override(
    cloud_name: RainChanceName,
    amount_name: RainAmountName,
) -> RainChanceName {
    match amount_name {
        RainAmountName::None => cloud_name,
        RainAmountName::Drizzle => {
            // Drizzle requires at least partly cloudy
            match cloud_name {
                RainChanceName::Clear => RainChanceName::PartlyCloudy,
                _ => cloud_name,
            }
        }
        RainAmountName::Rain => {
            // Heavy rain requires at least overcast
            match cloud_name {
                RainChanceName::Clear | RainChanceName::PartlyCloudy => RainChanceName::Overcast,
                _ => cloud_name,
            }
        }
    }
}

impl Icon for Precipitation {
    fn get_icon_name(&self) -> String {
        RainAmountIcon::RainAmount.to_string()
    }
}

impl Icon for DailyForecast {
    fn get_icon_name(&self) -> String {
        if let Some(ref precip) = self.precipitation {
            // Determine cloud coverage from cloud_cover data if available, otherwise fall back to precipitation chance
            let chance_name = if let Some(cloud_cover) = self.cloud_cover {
                cloud_cover_to_name(cloud_cover)
            } else {
                precip.chance_to_name()
            };

            let amount_name = precip.amount_to_name(false);

            // Apply precipitation override: ensure heavy rain requires adequate cloud cover
            // Note: After override, Clear can only occur with amount_name = None
            let adjusted_chance_name = apply_precipitation_override(chance_name, amount_name);

            format!("{adjusted_chance_name}{}{amount_name}.svg", DayNight::Day)
        } else {
            // Default to clear day if no precipitation data
            format!("{}{}.svg", RainChanceName::Clear, DayNight::Day)
        }
    }
}

impl Icon for HourlyForecast {
    fn get_icon_name(&self) -> String {
        // Determine cloud coverage from cloud_cover data if available, otherwise fall back to precipitation chance
        let chance_name = if let Some(cloud_cover) = self.cloud_cover {
            cloud_cover_to_name(cloud_cover)
        } else {
            self.precipitation.chance_to_name()
        };

        let amount_name = self.precipitation.amount_to_name(true);
        let day_night = if self.is_night {
            DayNight::Night
        } else {
            DayNight::Day
        };

        // Apply precipitation override: ensure heavy rain requires adequate cloud cover
        // Note: After override, Clear can only occur with amount_name = None
        let adjusted_chance_name = apply_precipitation_override(chance_name, amount_name);

        let mut icon_name = format!("{adjusted_chance_name}{day_night}{amount_name}.svg");

        if CONFIG.render_options.use_moon_phase_instead_of_clear_night
            && icon_name.ends_with(&format!("{}{}.svg", RainChanceName::Clear, DayNight::Night))
        {
            logger::detail("Using moon phase icon instead of clear night");
            icon_name = get_moon_phase_icon_name().to_string();
        }

        icon_name
    }
}

/// Helper struct for UV index icon selection
pub struct UVIndex(pub u16);

impl Icon for UVIndex {
    fn get_icon_name(&self) -> String {
        match self.0 {
            0 => UVIndexIcon::None,
            1..=2 => UVIndexIcon::Low,
            3..=5 => UVIndexIcon::Moderate,
            6..=7 => UVIndexIcon::High,
            8..=10 => UVIndexIcon::VeryHigh,
            11.. => UVIndexIcon::Extreme,
        }
        .to_string()
    }
}

/// Helper struct for relative humidity icon selection
pub struct RelativeHumidity(pub u16);

impl Icon for RelativeHumidity {
    fn get_icon_name(&self) -> String {
        match self.0 {
            0..=40 => HumidityIconName::Humidity.to_string(),
            41..=70 => HumidityIconName::HumidityPlus.to_string(),
            71.. => HumidityIconName::HumidityPlusPlus.to_string(),
        }
    }
}
