use super::models::{DailyForecast, HourlyForecast, Precipitation, Wind};
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

impl Icon for Precipitation {
    fn get_icon_name(&self) -> String {
        RainAmountIcon::RainAmount.to_string()
    }
}

impl Icon for DailyForecast {
    fn get_icon_name(&self) -> String {
        if let Some(ref precip) = self.precipitation {
            let chance_name = precip.chance_to_name();
            let amount_name = precip.amount_to_name(false);

            // Clear skies should never have precipitation amounts
            // (clear-day-drizzle.svg and clear-day-rain.svg don't exist)
            let final_amount = if matches!(chance_name, RainChanceName::Clear) {
                RainAmountName::None
            } else {
                amount_name
            };

            format!("{chance_name}{}{final_amount}.svg", DayNight::Day)
        } else {
            // Default to clear day if no precipitation data
            format!("{}{}.svg", RainChanceName::Clear, DayNight::Day)
        }
    }
}

impl Icon for HourlyForecast {
    fn get_icon_name(&self) -> String {
        let chance_name = self.precipitation.chance_to_name();
        let amount_name = self.precipitation.amount_to_name(true);
        let day_night = if self.is_night {
            DayNight::Night
        } else {
            DayNight::Day
        };

        // Clear skies should never have precipitation amounts
        // (clear-day-drizzle.svg, clear-night-drizzle.svg, etc. don't exist)
        let final_amount = if matches!(chance_name, RainChanceName::Clear) {
            RainAmountName::None
        } else {
            amount_name
        };

        let mut icon_name = format!("{chance_name}{day_night}{final_amount}.svg");

        if CONFIG.render_options.use_moon_phase_instead_of_clear_night
            && icon_name.ends_with(&format!("{}{}.svg", RainChanceName::Clear, DayNight::Night))
        {
            println!("'use_moon_phase_instead_of_clear_night' is set to true, using moon phase icon instead of clear night");
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
