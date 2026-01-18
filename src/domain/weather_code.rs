//! WMO Weather Interpretation Codes (WW) mapping to weather icons
//!
//! This module provides a structured mapping from WMO weather codes (0-99)
//! to appropriate weather icon names. The WMO codes provide more precise
//! weather classification than deriving conditions from precipitation/cloud data.
//!
//! Reference: https://open-meteo.com/en/docs#weathervariables

use std::fmt;

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
    pub fn to_icon_name(&self, is_night: bool) -> String {
        let day_night = if is_night { "night" } else { "day" };

        match self {
            // Clear sky conditions (0-3)
            Self::ClearSky => format!("clear-{day_night}.svg"),
            Self::MainlyClear => format!("partly-cloudy-{day_night}.svg"),
            Self::PartlyCloudy => format!("partly-cloudy-{day_night}.svg"),
            Self::Overcast => format!("overcast-{day_night}.svg"),

            // Fog (45, 48)
            Self::Fog | Self::RimeFog => "fog.svg".to_string(),

            // Drizzle (51, 53, 55) - Light → PartlyCloudy, Moderate → Overcast, Dense → Overcast
            Self::DrizzleLight => format!("partly-cloudy-{day_night}-drizzle.svg"),
            Self::DrizzleModerate | Self::DrizzleDense => {
                format!("overcast-{day_night}-drizzle.svg")
            }

            // Freezing drizzle (56, 57) - Use sleet as closest match
            Self::FreezingDrizzleLight => format!("partly-cloudy-{day_night}-sleet.svg"),
            Self::FreezingDrizzleDense => format!("overcast-{day_night}-sleet.svg"),

            // Rain (61, 63, 65) - Slight → PartlyCloudy, Moderate → Overcast, Heavy → Extreme
            Self::RainSlight => format!("partly-cloudy-{day_night}-rain.svg"),
            Self::RainModerate => format!("overcast-{day_night}-rain.svg"),
            Self::RainHeavy => format!("extreme-{day_night}-rain.svg"),

            // Freezing rain (66, 67) - Use sleet as closest match
            Self::FreezingRainLight => format!("partly-cloudy-{day_night}-sleet.svg"),
            Self::FreezingRainHeavy => format!("extreme-{day_night}-sleet.svg"),

            // Snow (71, 73, 75, 77) - Slight → PartlyCloudy, Moderate → Overcast, Heavy → Extreme
            Self::SnowSlight => format!("partly-cloudy-{day_night}-snow.svg"),
            Self::SnowModerate => format!("overcast-{day_night}-snow.svg"),
            Self::SnowHeavy => format!("extreme-{day_night}-snow.svg"),
            Self::SnowGrains => format!("overcast-{day_night}-snow.svg"), // No specific icon

            // Rain showers (80, 81, 82) - Slight → PartlyCloudy, Moderate → Overcast, Violent → Extreme
            Self::RainShowersSlight => format!("partly-cloudy-{day_night}-rain.svg"),
            Self::RainShowersModerate => format!("overcast-{day_night}-rain.svg"),
            Self::RainShowersViolent => format!("extreme-{day_night}-rain.svg"),

            // Snow showers (85, 86) - Slight → PartlyCloudy, Heavy → Extreme
            Self::SnowShowersSlight => format!("partly-cloudy-{day_night}-snow.svg"),
            Self::SnowShowersHeavy => format!("extreme-{day_night}-snow.svg"),

            // Thunderstorms (95, 96, 99)
            Self::Thunderstorm => format!("thunderstorms-{day_night}.svg"),
            Self::ThunderstormHailSlight => format!("thunderstorms-{day_night}-rain.svg"), // Hail shown as heavy rain
            Self::ThunderstormHailHeavy => format!("thunderstorms-{day_night}-extreme-rain.svg"),

            // Fallback for unknown codes
            Self::Unknown => format!("overcast-{day_night}.svg"),
        }
    }

    /// Check if this weather code represents precipitation
    pub fn is_precipitation(&self) -> bool {
        matches!(
            self,
            Self::DrizzleLight
                | Self::DrizzleModerate
                | Self::DrizzleDense
                | Self::FreezingDrizzleLight
                | Self::FreezingDrizzleDense
                | Self::RainSlight
                | Self::RainModerate
                | Self::RainHeavy
                | Self::FreezingRainLight
                | Self::FreezingRainHeavy
                | Self::SnowSlight
                | Self::SnowModerate
                | Self::SnowHeavy
                | Self::SnowGrains
                | Self::RainShowersSlight
                | Self::RainShowersModerate
                | Self::RainShowersViolent
                | Self::SnowShowersSlight
                | Self::SnowShowersHeavy
                | Self::Thunderstorm
                | Self::ThunderstormHailSlight
                | Self::ThunderstormHailHeavy
        )
    }

    /// Check if this weather code represents snow
    pub fn is_snow(&self) -> bool {
        matches!(
            self,
            Self::SnowSlight
                | Self::SnowModerate
                | Self::SnowHeavy
                | Self::SnowGrains
                | Self::SnowShowersSlight
                | Self::SnowShowersHeavy
        )
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
        assert_eq!(
            WmoWeatherCode::ClearSky.to_icon_name(false),
            "clear-day.svg"
        );
        assert_eq!(
            WmoWeatherCode::PartlyCloudy.to_icon_name(false),
            "partly-cloudy-day.svg"
        );
        assert_eq!(WmoWeatherCode::Fog.to_icon_name(false), "fog.svg");
        assert_eq!(
            WmoWeatherCode::RainModerate.to_icon_name(false),
            "overcast-day-rain.svg"
        );
        assert_eq!(
            WmoWeatherCode::SnowHeavy.to_icon_name(false),
            "extreme-day-snow.svg"
        );
        assert_eq!(
            WmoWeatherCode::Thunderstorm.to_icon_name(false),
            "thunderstorms-day.svg"
        );
    }

    #[test]
    fn test_icon_name_generation_night() {
        assert_eq!(
            WmoWeatherCode::ClearSky.to_icon_name(true),
            "clear-night.svg"
        );
        assert_eq!(
            WmoWeatherCode::RainHeavy.to_icon_name(true),
            "extreme-night-rain.svg"
        );
        assert_eq!(
            WmoWeatherCode::Thunderstorm.to_icon_name(true),
            "thunderstorms-night.svg"
        );
    }

    #[test]
    fn test_intensity_gradation() {
        // Light → PartlyCloudy
        assert!(WmoWeatherCode::DrizzleLight
            .to_icon_name(false)
            .contains("partly-cloudy"));
        assert!(WmoWeatherCode::RainSlight
            .to_icon_name(false)
            .contains("partly-cloudy"));
        assert!(WmoWeatherCode::SnowSlight
            .to_icon_name(false)
            .contains("partly-cloudy"));

        // Moderate → Overcast
        assert!(WmoWeatherCode::DrizzleModerate
            .to_icon_name(false)
            .contains("overcast"));
        assert!(WmoWeatherCode::RainModerate
            .to_icon_name(false)
            .contains("overcast"));
        assert!(WmoWeatherCode::SnowModerate
            .to_icon_name(false)
            .contains("overcast"));

        // Heavy → Extreme
        assert!(WmoWeatherCode::RainHeavy
            .to_icon_name(false)
            .contains("extreme"));
        assert!(WmoWeatherCode::SnowHeavy
            .to_icon_name(false)
            .contains("extreme"));
    }

    #[test]
    fn test_is_precipitation() {
        assert!(WmoWeatherCode::RainModerate.is_precipitation());
        assert!(WmoWeatherCode::SnowSlight.is_precipitation());
        assert!(WmoWeatherCode::Thunderstorm.is_precipitation());
        assert!(!WmoWeatherCode::ClearSky.is_precipitation());
        assert!(!WmoWeatherCode::Fog.is_precipitation());
    }

    #[test]
    fn test_is_snow() {
        assert!(WmoWeatherCode::SnowSlight.is_snow());
        assert!(WmoWeatherCode::SnowShowersHeavy.is_snow());
        assert!(!WmoWeatherCode::RainModerate.is_snow());
        assert!(!WmoWeatherCode::FreezingRainLight.is_snow());
    }

    #[test]
    fn test_display() {
        assert_eq!(WmoWeatherCode::Thunderstorm.to_string(), "Thunderstorm");
        assert_eq!(WmoWeatherCode::RainModerate.to_string(), "Moderate rain");
        assert_eq!(WmoWeatherCode::Fog.to_string(), "Fog");
    }
}
