//! WMO Weather Interpretation Codes (WW) mapping to weather icons
//!
//! This module provides a structured mapping from WMO weather codes (0-99)
//! to appropriate weather icon names. The WMO codes provide more precise
//! weather classification than deriving conditions from precipitation/cloud data.
//!
//! Reference: https://open-meteo.com/en/docs#weathervariables

use std::fmt;

use crate::weather::icons::{DayNight, PrecipitationChanceName, PrecipitationKind};

/// WMO Weather Interpretation Codes
///
/// These codes are provided by Open-Meteo API and represent the current
/// weather condition as a single categorical value rather than separate
/// precipitation/cloud/visibility data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WmoWeatherCode {
    /// Code 0: Clear sky
    ClearSky,
    /// Code 1: Mainly clear
    MainlyClear,
    /// Code 2: Partly cloudy
    PartlyCloudy,
    /// Code 3: Overcast
    Overcast,
    /// Code 45: Fog
    Fog,
    /// Code 48: Depositing rime fog
    RimeFog,
    /// Code 51: Light drizzle
    DrizzleLight,
    /// Code 53: Moderate drizzle
    DrizzleModerate,
    /// Code 55: Dense drizzle
    DrizzleDense,
    /// Code 56: Light freezing drizzle
    FreezingDrizzleLight,
    /// Code 57: Dense freezing drizzle
    FreezingDrizzleDense,
    /// Code 61: Slight rain
    RainSlight,
    /// Code 63: Moderate rain
    RainModerate,
    /// Code 65: Heavy rain
    RainHeavy,
    /// Code 66: Light freezing rain
    FreezingRainLight,
    /// Code 67: Heavy freezing rain
    FreezingRainHeavy,
    /// Code 71: Slight snow fall
    SnowSlight,
    /// Code 73: Moderate snow fall
    SnowModerate,
    /// Code 75: Heavy snow fall
    SnowHeavy,
    /// Code 77: Snow grains
    SnowGrains,
    /// Code 80: Slight rain showers
    RainShowersSlight,
    /// Code 81: Moderate rain showers
    RainShowersModerate,
    /// Code 82: Violent rain showers
    RainShowersViolent,
    /// Code 85: Slight snow showers
    SnowShowersSlight,
    /// Code 86: Heavy snow showers
    SnowShowersHeavy,
    /// Code 95: Thunderstorm (slight or moderate)
    Thunderstorm,
    /// Code 96: Thunderstorm with slight hail
    ThunderstormHailSlight,
    /// Code 99: Thunderstorm with heavy hail
    ThunderstormHailHeavy,
    /// Unknown or unsupported code
    Unknown,
}

impl From<u8> for WmoWeatherCode {
    fn from(code: u8) -> Self {
        match code {
            0 => Self::ClearSky,
            1 => Self::MainlyClear,
            2 => Self::PartlyCloudy,
            3 => Self::Overcast,
            45 => Self::Fog,
            48 => Self::RimeFog,
            51 => Self::DrizzleLight,
            53 => Self::DrizzleModerate,
            55 => Self::DrizzleDense,
            56 => Self::FreezingDrizzleLight,
            57 => Self::FreezingDrizzleDense,
            61 => Self::RainSlight,
            63 => Self::RainModerate,
            65 => Self::RainHeavy,
            66 => Self::FreezingRainLight,
            67 => Self::FreezingRainHeavy,
            71 => Self::SnowSlight,
            73 => Self::SnowModerate,
            75 => Self::SnowHeavy,
            77 => Self::SnowGrains,
            80 => Self::RainShowersSlight,
            81 => Self::RainShowersModerate,
            82 => Self::RainShowersViolent,
            85 => Self::SnowShowersSlight,
            86 => Self::SnowShowersHeavy,
            95 => Self::Thunderstorm,
            96 => Self::ThunderstormHailSlight,
            99 => Self::ThunderstormHailHeavy,
            _ => Self::Unknown,
        }
    }
}

impl WmoWeatherCode {
    /// Convert WMO weather code to icon filename
    ///
    /// Uses recommended intensity gradation:
    /// - Light intensity → partly-cloudy
    /// - Moderate intensity → overcast
    /// - Heavy/Violent intensity → extreme
    ///
    /// # Arguments
    /// * `is_night` - Whether it's nighttime (affects day/night suffix)
    ///
    /// # Returns
    /// Icon filename (e.g., "partly-cloudy-day-rain.svg", "thunderstorms-night.svg")
    pub fn icon_name(&self, is_night: bool) -> String {
        let day_night = if is_night {
            DayNight::Night
        } else {
            DayNight::Day
        };

        match self {
            // Clear sky conditions (0-3)
            Self::ClearSky => format!("{}{day_night}.svg", PrecipitationChanceName::Clear),
            Self::MainlyClear => format!("mainly-clear{day_night}.svg"),
            Self::PartlyCloudy => {
                format!("{}{day_night}.svg", PrecipitationChanceName::PartlyCloudy)
            }
            Self::Overcast => format!("{}{day_night}.svg", PrecipitationChanceName::Overcast),

            // Fog (45, 48)
            Self::Fog | Self::RimeFog => format!("fog{day_night}.svg"),

            // Drizzle (51, 53, 55) - Light → PartlyCloudy, Moderate/Dense → Overcast
            Self::DrizzleLight => {
                format!(
                    "{}{day_night}{}.svg",
                    PrecipitationChanceName::PartlyCloudy,
                    PrecipitationKind::Drizzle
                )
            }
            Self::DrizzleModerate => {
                format!(
                    "{}{day_night}{}.svg",
                    PrecipitationChanceName::Overcast,
                    PrecipitationKind::Drizzle
                )
            }
            Self::DrizzleDense => {
                format!(
                    "{}{day_night}{}.svg",
                    PrecipitationChanceName::Extreme,
                    PrecipitationKind::Drizzle
                )
            }

            // Freezing drizzle (56, 57) - Use sleet as closest match
            Self::FreezingDrizzleLight => {
                format!(
                    "{}{day_night}{}.svg",
                    PrecipitationChanceName::PartlyCloudy,
                    PrecipitationKind::Sleet
                )
            }
            Self::FreezingDrizzleDense => {
                format!(
                    "{}{day_night}{}.svg",
                    PrecipitationChanceName::Overcast,
                    PrecipitationKind::Sleet
                )
            }

            // Rain (61, 63, 65) - Slight → PartlyCloudy, Moderate → Overcast, Heavy → Extreme
            Self::RainSlight => {
                format!(
                    "{}{day_night}{}.svg",
                    PrecipitationChanceName::PartlyCloudy,
                    PrecipitationKind::Rain
                )
            }
            Self::RainModerate => {
                format!(
                    "{}{day_night}{}.svg",
                    PrecipitationChanceName::Overcast,
                    PrecipitationKind::Rain
                )
            }
            Self::RainHeavy => {
                format!(
                    "{}{day_night}{}.svg",
                    PrecipitationChanceName::Extreme,
                    PrecipitationKind::Rain
                )
            }

            // Freezing rain (66, 67) - Use sleet as closest match
            Self::FreezingRainLight => {
                format!(
                    "{}{day_night}{}.svg",
                    PrecipitationChanceName::PartlyCloudy,
                    PrecipitationKind::Sleet
                )
            }
            Self::FreezingRainHeavy => {
                format!(
                    "{}{day_night}{}.svg",
                    PrecipitationChanceName::Extreme,
                    PrecipitationKind::Sleet
                )
            }

            // Snow (71, 73, 75, 77) - Slight → PartlyCloudy, Moderate → Overcast, Heavy → Extreme
            Self::SnowSlight => {
                format!(
                    "{}{day_night}{}.svg",
                    PrecipitationChanceName::PartlyCloudy,
                    PrecipitationKind::Snow
                )
            }
            Self::SnowModerate => {
                format!(
                    "{}{day_night}{}.svg",
                    PrecipitationChanceName::Overcast,
                    PrecipitationKind::Snow
                )
            }
            Self::SnowHeavy => {
                format!(
                    "{}{day_night}{}.svg",
                    PrecipitationChanceName::Extreme,
                    PrecipitationKind::Snow
                )
            }
            Self::SnowGrains => format!("overcast{day_night}-snow-grains.svg"),

            // Rain showers (80, 81, 82) - Slight → PartlyCloudy, Moderate → Overcast, Violent → Extreme
            Self::RainShowersSlight => {
                format!(
                    "{}{day_night}{}.svg",
                    PrecipitationChanceName::PartlyCloudy,
                    PrecipitationKind::Rain
                )
            }
            Self::RainShowersModerate => {
                format!(
                    "{}{day_night}{}.svg",
                    PrecipitationChanceName::Overcast,
                    PrecipitationKind::Rain
                )
            }
            Self::RainShowersViolent => {
                format!(
                    "{}{day_night}{}.svg",
                    PrecipitationChanceName::Extreme,
                    PrecipitationKind::Rain
                )
            }

            // Snow showers (85, 86) - Slight → PartlyCloudy, Heavy → Extreme
            Self::SnowShowersSlight => {
                format!(
                    "{}{day_night}{}.svg",
                    PrecipitationChanceName::PartlyCloudy,
                    PrecipitationKind::Snow
                )
            }
            Self::SnowShowersHeavy => {
                format!(
                    "{}{day_night}{}.svg",
                    PrecipitationChanceName::Extreme,
                    PrecipitationKind::Snow
                )
            }

            // Thunderstorms (95, 96, 99)
            Self::Thunderstorm => format!("thunderstorms{day_night}.svg"),
            Self::ThunderstormHailSlight => {
                format!("thunderstorms{day_night}{}.svg", PrecipitationKind::Hail)
            }
            Self::ThunderstormHailHeavy => {
                format!(
                    "thunderstorms{day_night}-{}{}.svg",
                    PrecipitationChanceName::Extreme,
                    PrecipitationKind::Hail
                )
            }

            // Fallback for unknown codes
            Self::Unknown => format!("{}{day_night}.svg", PrecipitationChanceName::Overcast),
        }
    }
}

impl fmt::Display for WmoWeatherCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            Self::ClearSky => "Clear sky",
            Self::MainlyClear => "Mainly clear",
            Self::PartlyCloudy => "Partly cloudy",
            Self::Overcast => "Overcast",
            Self::Fog => "Fog",
            Self::RimeFog => "Depositing rime fog",
            Self::DrizzleLight => "Light drizzle",
            Self::DrizzleModerate => "Moderate drizzle",
            Self::DrizzleDense => "Dense drizzle",
            Self::FreezingDrizzleLight => "Light freezing drizzle",
            Self::FreezingDrizzleDense => "Dense freezing drizzle",
            Self::RainSlight => "Slight rain",
            Self::RainModerate => "Moderate rain",
            Self::RainHeavy => "Heavy rain",
            Self::FreezingRainLight => "Light freezing rain",
            Self::FreezingRainHeavy => "Heavy freezing rain",
            Self::SnowSlight => "Slight snow",
            Self::SnowModerate => "Moderate snow",
            Self::SnowHeavy => "Heavy snow",
            Self::SnowGrains => "Snow grains",
            Self::RainShowersSlight => "Slight rain showers",
            Self::RainShowersModerate => "Moderate rain showers",
            Self::RainShowersViolent => "Violent rain showers",
            Self::SnowShowersSlight => "Slight snow showers",
            Self::SnowShowersHeavy => "Heavy snow showers",
            Self::Thunderstorm => "Thunderstorm",
            Self::ThunderstormHailSlight => "Thunderstorm with slight hail",
            Self::ThunderstormHailHeavy => "Thunderstorm with heavy hail",
            Self::Unknown => "Unknown weather",
        };
        write!(f, "{}", description)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wmo_code_conversion() {
        assert_eq!(WmoWeatherCode::from(0), WmoWeatherCode::ClearSky);
        assert_eq!(WmoWeatherCode::from(45), WmoWeatherCode::Fog);
        assert_eq!(WmoWeatherCode::from(71), WmoWeatherCode::SnowSlight);
        assert_eq!(WmoWeatherCode::from(95), WmoWeatherCode::Thunderstorm);
        assert_eq!(WmoWeatherCode::from(255), WmoWeatherCode::Unknown);
    }

    #[test]
    fn test_icon_name_generation_day() {
        assert_eq!(WmoWeatherCode::ClearSky.icon_name(false), "clear-day.svg");
        assert_eq!(
            WmoWeatherCode::PartlyCloudy.icon_name(false),
            "partly-cloudy-day.svg"
        );
        assert_eq!(WmoWeatherCode::Fog.icon_name(false), "fog-day.svg");
        assert_eq!(
            WmoWeatherCode::RainModerate.icon_name(false),
            "overcast-day-rain.svg"
        );
        assert_eq!(
            WmoWeatherCode::SnowHeavy.icon_name(false),
            "extreme-day-snow.svg"
        );
        assert_eq!(
            WmoWeatherCode::Thunderstorm.icon_name(false),
            "thunderstorms-day.svg"
        );
        assert_eq!(
            WmoWeatherCode::ThunderstormHailSlight.icon_name(false),
            "thunderstorms-day-hail.svg"
        );
        assert_eq!(
            WmoWeatherCode::ThunderstormHailHeavy.icon_name(false),
            "thunderstorms-day-extreme-hail.svg"
        );
    }

    #[test]
    fn test_icon_name_generation_night() {
        assert_eq!(WmoWeatherCode::ClearSky.icon_name(true), "clear-night.svg");
        assert_eq!(WmoWeatherCode::Fog.icon_name(true), "fog-night.svg");
        assert_eq!(
            WmoWeatherCode::RainHeavy.icon_name(true),
            "extreme-night-rain.svg"
        );
        assert_eq!(
            WmoWeatherCode::Thunderstorm.icon_name(true),
            "thunderstorms-night.svg"
        );
        assert_eq!(
            WmoWeatherCode::ThunderstormHailSlight.icon_name(true),
            "thunderstorms-night-hail.svg"
        );
        assert_eq!(
            WmoWeatherCode::ThunderstormHailHeavy.icon_name(true),
            "thunderstorms-night-extreme-hail.svg"
        );
    }

    #[test]
    fn test_intensity_gradation() {
        // Light → PartlyCloudy
        assert!(WmoWeatherCode::DrizzleLight
            .icon_name(false)
            .contains("partly-cloudy"));
        assert!(WmoWeatherCode::RainSlight
            .icon_name(false)
            .contains("partly-cloudy"));
        assert!(WmoWeatherCode::SnowSlight
            .icon_name(false)
            .contains("partly-cloudy"));

        // Moderate → Overcast
        assert!(WmoWeatherCode::DrizzleModerate
            .icon_name(false)
            .contains("overcast"));
        assert!(WmoWeatherCode::RainModerate
            .icon_name(false)
            .contains("overcast"));
        assert!(WmoWeatherCode::SnowModerate
            .icon_name(false)
            .contains("overcast"));

        // Heavy → Extreme
        assert!(WmoWeatherCode::RainHeavy
            .icon_name(false)
            .contains("extreme"));
        assert!(WmoWeatherCode::SnowHeavy
            .icon_name(false)
            .contains("extreme"));
    }

    #[test]
    fn test_display() {
        assert_eq!(WmoWeatherCode::Thunderstorm.to_string(), "Thunderstorm");
        assert_eq!(WmoWeatherCode::RainModerate.to_string(), "Moderate rain");
        assert_eq!(WmoWeatherCode::Fog.to_string(), "Fog");
    }
}
