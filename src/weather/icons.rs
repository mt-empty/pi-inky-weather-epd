use serde::Deserialize;
use strum_macros::Display;

use crate::apis::bom::models::*;
use crate::constants::NOT_AVAILABLE_ICON;
use crate::weather::utils::get_moon_phase_icon_path;
use crate::CONFIG;

#[derive(Deserialize, Debug, Display)]
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

#[derive(Deserialize, Debug, Display)]
pub enum RainAmountName {
    #[strum(to_string = "")]
    None,
    #[strum(to_string = "-drizzle")]
    Drizzle,
    #[strum(to_string = "-rain")]
    Rain,
}

#[derive(Deserialize, Debug, Display)]
pub enum DayNight {
    #[strum(to_string = "-day")]
    Day,
    #[strum(to_string = "-night")]
    Night,
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
///
/// - `rain_amount_to_name(amount: u32) -> String`
///
///   Converts a given rain amount (in millimetres) to a corresponding name.
///   This method should not be part of this trait and needs to be refactored.
///
///   - `amount: u32` - The amount of rain in millimetres.
///   - Returns a `String` representing the rain amount name:
///     - `""` for 0 to 2 mm
///     - `"Drizzle"` for 3 to 20 mm
///     - `"Rain"` for 21 mm and above
///
/// - `rain_chance_to_name(chance: u32) -> String`
///
///   Converts a given rain chance (in percentage) to a corresponding name.
///   This method should not be part of this trait and needs to be refactored.
///
///   - `chance: u32` - The chance of rain in percentage.
///   - Returns a `String` representing the rain chance name:
///     - `"Clear"` for 0 to 25%
///     - `"PartlyCloudy"` for 26 to 50%
///     - `"Overcast"` for 51 to 75%
///     - `"Extreme"` for 76% and above
pub trait Icon {
    /// Returns the name of the icon as a `String`.
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

    /// Converts a given rain amount (in millimetres) to a corresponding name.
    ///
    /// # Arguments
    ///
    /// * `amount` - The amount of rain.
    ///
    /// # Returns
    ///
    /// * A `String` representing the name corresponding to the rain amount.
    fn rain_amount_to_name(mut amount: u32, is_hourly: bool) -> String {
        // TODO: this should be moved to a more appropriate place
        if is_hourly {
            amount *= 24;
        }
        match amount {
            0..=2 => RainAmountName::None.to_string(),
            3..=20 => RainAmountName::Drizzle.to_string(),
            21.. => RainAmountName::Rain.to_string(),
        }
    }

    /// Converts a given rain chance (in percentage) to a corresponding name as a `String`.
    ///
    /// # Arguments
    ///
    /// * `chance` - The chance of rain.
    ///
    /// # Returns
    ///
    /// * A `String` representing the name corresponding to the rain chance.
    fn rain_chance_to_name(chance: u32) -> String {
        // TODO: this should be moved to a more appropriate place
        match chance {
            0..=25 => RainChanceName::Clear.to_string(),
            26..=50 => RainChanceName::PartlyCloudy.to_string(),
            51..=75 => RainChanceName::Overcast.to_string(),
            76.. => RainChanceName::Extreme.to_string(),
        }
    }
}

impl Icon for Wind {
    fn get_icon_name(&self) -> String {
        let icon = match self.speed_kilometre {
            0.0..=20.0 => "wind.svg",
            20.1..=40.0 => "umbrella-wind.svg",
            40.1.. => "umbrella-wind-alt.svg",
            _ => NOT_AVAILABLE_ICON,
        };
        icon.to_string()
    }
}

impl Icon for RainAmount {
    fn get_icon_name(&self) -> String {
        "raindrop-measure.svg".to_string()
    }
}

impl Icon for UV {
    fn get_icon_name(&self) -> String {
        match self.max_index {
            Some(_index) => "uv-index.svg".to_string(),
            // Some(index) => match index {
            //     0.. => "uv-index.svg".to_string(),
            //     // 1..=11 => format!("uv-index-{}.svg", index),
            //     _ => NOT_AVAILABLE_ICON.to_string(),
            // },
            None => NOT_AVAILABLE_ICON.to_string(),
        }
    }
}

impl Icon for DailyEntry {
    fn get_icon_name(&self) -> String {
        // TODO: this might generate invalid icon names, like clear-day/night-drzzile/rain
        let icon_name = format!(
            "{}{}{}.svg",
            DailyEntry::rain_chance_to_name(self.rain.as_ref().unwrap().chance.unwrap_or(0)),
            DayNight::Day,
            DailyEntry::rain_amount_to_name(
                self.rain
                    .as_ref()
                    .unwrap()
                    .amount
                    .min
                    .unwrap_or(0.0)
                    .round() as u32,
                false
            )
        );
        icon_name
    }
}

type RelativeHumidity = f64;

impl Icon for RelativeHumidity {
    fn get_icon_name(&self) -> String {
        "humidity.svg".to_string()
    }
}

impl Icon for HourlyForecast {
    fn get_icon_name(&self) -> String {
        let mut icon_name = format!(
            "{}{}{}.svg",
            HourlyForecast::rain_chance_to_name(self.rain.chance.unwrap_or(0)),
            if self.is_night {
                DayNight::Night.to_string()
            } else {
                DayNight::Day.to_string()
            },
            HourlyForecast::rain_amount_to_name(
                self.rain.amount.min.unwrap_or(0.0).round() as u32,
                true
            )
        );

        if CONFIG.render_options.use_moon_phase_instead_of_clear_night
            && icon_name.ends_with(&format!("{}{}.svg", RainChanceName::Clear, DayNight::Night))
        {
            println!("'use_moon_phase_instead_of_clear_night' is set to true, using moon phase icon instead of clear night");
            icon_name = get_moon_phase_icon_path().to_string();
        }

        icon_name
    }
}
