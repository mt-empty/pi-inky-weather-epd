use strum_macros::Display;

use crate::apis::bom::models::*;
use crate::weather::utils::get_moon_phase_icon_name;
use crate::CONFIG;

#[derive(Debug, Display)]
pub enum RainChanceName {
    #[strum(to_string = "clear")]
    Clear,
    #[strum(to_string = "partly-cloudy")]
    PartlyCloudy,
    #[strum(to_string = "overcast")]
    Overcast,
    #[strum(to_string = "extreme")]
    Extreme,
}

#[derive(Debug, Display)]
pub enum RainAmountName {
    #[strum(to_string = "")]
    None,
    #[strum(to_string = "-drizzle")]
    Drizzle,
    #[strum(to_string = "-rain")]
    Rain,
}

#[derive(Debug, Display)]
pub enum DayNight {
    #[strum(to_string = "-day")]
    Day,
    #[strum(to_string = "-night")]
    Night,
}

#[derive(Debug, Display)]
pub enum WindIconName {
    #[strum(to_string = "wind.svg")]
    Wind,
    #[strum(to_string = "umbrella-wind.svg")]
    UmbrellaWind,
    #[strum(to_string = "umbrella-wind-alt.svg")]
    UmbrellaWindAlt,
}

#[derive(Debug, Display)]
pub enum HumidityIconName {
    #[strum(to_string = "humidity.svg")]
    Humidity,
    #[strum(to_string = "humidity-plus.svg")]
    HumidityPlus,
    #[strum(to_string = "humidity-plus-plus.svg")]
    HumidityPlusPlus,
}

#[derive(Debug, Display)]
pub enum SunPositionIconName {
    #[strum(to_string = "sunrise.svg")]
    Sunrise,
    #[strum(to_string = "sunset.svg")]
    Sunset,
}

#[derive(Debug, Display)]
pub enum NotAvailableIcon {
    #[strum(to_string = "not-available.svg")]
    NotAvailable,
}

#[derive(Debug, Display)]
pub enum HumidityIcon {
    #[strum(to_string = "humidity.svg")]
    Humidity,
}

#[derive(Debug, Display)]
pub enum RainAmountIcon {
    #[strum(to_string = "raindrop-measure.svg")]
    RainAmount,
}

#[derive(Debug, Display)]
pub enum UVIndexIcon {
    #[strum(to_string = "uv-index-none.svg")]
    None,
    #[strum(to_string = "uv-index-low.svg")]
    Low,
    #[strum(to_string = "uv-index-moderate.svg")]
    Moderate,
    #[strum(to_string = "uv-index-high.svg")]
    High,
    #[strum(to_string = "uv-index-very-high.svg")]
    VeryHigh,
    #[strum(to_string = "uv-index-extreme.svg")]
    Extreme,
}

/// A trait representing an icon with methods to get its name and path.
///
/// # Methods
///
/// - `get_icon_name(&self) -> String`
///
///   Returns the name of the icon as a `String`.
///
/// - `get_icon_path(&self) -> String`
///
///   Returns the full path to the icon as a `String`. The path is constructed
///   by concatenating the `svg_icons_directory` from the `CONFIG` with the icon name.
pub trait Icon {
    /// Returns the name of the icon
    fn get_icon_name(&self) -> String;

    /// Returns the path of the icon as a `String`.
    /// The path is constructed using the `svg_icons_directory` from the configuration
    /// and the icon name obtained from `get_icon_name`.
    fn get_icon_path(&self) -> String {
        format!(
            "{}{}",
            CONFIG.misc.svg_icons_directory,
            self.get_icon_name()
        )
    }
}

/// Methods for converting rain data to icon name enums.
impl Rain {
    /// Converts the rain amount to a corresponding `RainAmountName`.
    ///
    /// # Arguments
    ///
    /// * `is_hourly` - If true, treats the rain amount as hourly and scales accordingly.
    ///
    /// # Returns
    ///
    /// * A `RainAmountName` variant representing the rain amount.
    fn rain_amount_to_name(&self, is_hourly: bool) -> RainAmountName {
        let mut median_rain = self.calculate_median_rain();

        if is_hourly {
            median_rain *= 24.0;
        }
        match median_rain {
            0.0..=2.0 => RainAmountName::None,
            3.0..=20.0 => RainAmountName::Drizzle,
            21.0.. => RainAmountName::Rain,
            _ => RainAmountName::None,
        }
    }

    pub fn calculate_median_rain(&self) -> f32 {
        let min_rain = self.amount.min.unwrap_or(0);
        let max_rain = self.amount.max.unwrap_or(min_rain);
        (min_rain + max_rain) as f32 / 2.0
    }

    /// Converts the rain chance (percentage) to a corresponding `RainChanceName`.
    ///
    /// # Returns
    ///
    /// * A `RainChanceName` variant representing the rain chance.
    fn rain_chance_to_name(&self) -> RainChanceName {
        match self.chance.unwrap_or(0) {
            0..=25 => RainChanceName::Clear,
            26..=50 => RainChanceName::PartlyCloudy,
            51..=75 => RainChanceName::Overcast,
            76.. => RainChanceName::Extreme,
        }
    }
}

impl Icon for Wind {
    fn get_icon_name(&self) -> String {
        match self.speed_kilometre {
            0..=20 => WindIconName::Wind,
            21..=40 => WindIconName::UmbrellaWind,
            41.. => WindIconName::UmbrellaWindAlt,
        }
        .to_string()
    }
}

impl Icon for RainAmount {
    fn get_icon_name(&self) -> String {
        RainAmountIcon::RainAmount.to_string()
    }
}

impl Icon for UV {
    fn get_icon_name(&self) -> String {
        match self.max_index {
            Some(index) => match index {
                0 => UVIndexIcon::None,
                1..=2 => UVIndexIcon::Low,
                3..=5 => UVIndexIcon::Moderate,
                6..=7 => UVIndexIcon::High,
                8..=10 => UVIndexIcon::VeryHigh,
                11.. => UVIndexIcon::Extreme,
            }
            .to_string(),
            None => NotAvailableIcon::NotAvailable.to_string(),
        }
    }
}

impl Icon for DailyEntry {
    fn get_icon_name(&self) -> String {
        // TODO: this might generate invalid icon names, like clear-day/night-drizzle/rain
        let icon_name = format!(
            "{}{}{}.svg",
            self.rain.as_ref().unwrap().rain_chance_to_name(),
            DayNight::Day,
            self.rain.as_ref().unwrap().rain_amount_to_name(false)
        );
        icon_name
    }
}

impl Icon for RelativeHumidity {
    fn get_icon_name(&self) -> String {
        match *self {
            0..=40 => HumidityIconName::Humidity.to_string(),
            41..=70 => HumidityIconName::HumidityPlus.to_string(),
            71.. => HumidityIconName::HumidityPlusPlus.to_string(),
        }
    }
}

impl Icon for HourlyForecast {
    fn get_icon_name(&self) -> String {
        let mut icon_name = format!(
            "{}{}{}.svg",
            self.rain.rain_chance_to_name(),
            if self.is_night {
                DayNight::Night
            } else {
                DayNight::Day
            },
            self.rain.rain_amount_to_name(true)
        );

        if CONFIG.render_options.use_moon_phase_instead_of_clear_night
            && icon_name.ends_with(&format!("{}{}.svg", RainChanceName::Clear, DayNight::Night))
        {
            println!("'use_moon_phase_instead_of_clear_night' is set to true, using moon phase icon instead of clear night");
            icon_name = get_moon_phase_icon_name().to_string();
        }

        icon_name
    }
}

impl Icon for SunPositionIconName {
    fn get_icon_name(&self) -> String {
        self.to_string()
    }
}

impl Icon for HumidityIconName {
    fn get_icon_name(&self) -> String {
        self.to_string()
    }
}
