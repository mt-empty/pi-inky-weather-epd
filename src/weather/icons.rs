use std::path::Path;

use strum_macros::Display;

use crate::CONFIG;

#[derive(Debug, Display, Copy, Clone)]
pub enum PrecipitationChanceName {
    #[strum(to_string = "clear")]
    Clear,
    #[strum(to_string = "partly-cloudy")]
    PartlyCloudy,
    #[strum(to_string = "overcast")]
    Overcast,
    #[strum(to_string = "extreme")]
    Extreme,
}

#[derive(Debug, Display, Copy, Clone, PartialEq)]
pub enum PrecipitationKind {
    #[strum(to_string = "-drizzle")]
    Drizzle,
    #[strum(to_string = "-rain")]
    Rain,
    #[strum(to_string = "-snow")]
    Snow,
    #[strum(to_string = "-sleet")]
    Sleet,
    #[strum(to_string = "-hail")]
    Hail,
}

#[derive(Debug, Display, Copy, Clone)]
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

#[derive(Debug, Display, Copy, Clone)]
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

impl From<u16> for UVIndexIcon {
    fn from(value: u16) -> Self {
        match value {
            0 => Self::None,
            1..=2 => Self::Low,
            3..=5 => Self::Moderate,
            6..=7 => Self::High,
            8..=10 => Self::VeryHigh,
            11.. => Self::Extreme,
        }
    }
}

impl UVIndexIcon {
    pub fn to_colour(self) -> &'static str {
        match self {
            Self::None => "white",
            Self::Low => "green",
            Self::Moderate => "yellow",
            Self::High => "orange",
            Self::VeryHigh => "red",
            Self::Extreme => "purple",
        }
    }
}

impl From<u16> for HumidityIconName {
    fn from(value: u16) -> Self {
        match value {
            0..=40 => Self::Humidity,
            41..=70 => Self::HumidityPlus,
            71.. => Self::HumidityPlusPlus,
        }
    }
}

/// A trait representing an icon with methods to get its name and path.
///
/// # Methods
///
/// - `icon_name(&self) -> String`
///
///   Returns the name of the icon as a `String`.
///
/// - `icon_path(&self) -> String`
///
///   Returns the full path to the icon as a `String`. The path is constructed
///   by concatenating the `svg_icons_directory` from the `CONFIG` with the icon name.
pub trait Icon {
    /// Returns the name of the icon
    fn icon_name(&self) -> String;

    /// Returns the path of the icon as a `String`.
    /// The path is constructed using the `svg_icons_directory` from the configuration
    /// and the icon name obtained from `icon_name`.
    fn icon_path(&self) -> String {
        CONFIG
            .misc
            .svg_icons_directory
            .join(Path::new(&self.icon_name()))
            .to_string_lossy()
            .to_string()
    }
}

impl Icon for SunPositionIconName {
    fn icon_name(&self) -> String {
        self.to_string()
    }
}

impl Icon for HumidityIconName {
    fn icon_name(&self) -> String {
        self.to_string()
    }
}

impl Icon for UVIndexIcon {
    fn icon_name(&self) -> String {
        self.to_string()
    }
}
