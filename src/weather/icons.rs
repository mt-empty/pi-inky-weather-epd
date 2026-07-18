use std::path::Path;

use chrono::NaiveDate;
use strum_macros::Display;

use crate::configs::settings::{DashboardSettings, RenderOptions};

/// The subset of settings icon rendering actually depends on — narrower than
/// `&DashboardSettings` so icon impls aren't coupled to config fields
/// (`api`, `release`, `colours`, `dev`) they never touch.
#[derive(Clone, Copy)]
pub struct IconContext<'a> {
    pub svg_icons_directory: &'a Path,
    pub render_options: &'a RenderOptions,
    /// Resolved once from the injected `Clock`, so moon-phase selection
    /// stays deterministic under `FixedClock`.
    pub today: NaiveDate,
}

impl<'a> IconContext<'a> {
    pub fn from_settings(settings: &'a DashboardSettings, today: NaiveDate) -> Self {
        Self {
            svg_icons_directory: &settings.misc.svg_icons_directory,
            render_options: &settings.render_options,
            today,
        }
    }
}

/// Shared fixture date for tests building an `IconContext`. Unused by most
/// tests (moon-phase selection is the only thing that reads `today`, and
/// `use_moon_phase_instead_of_clear_night` is `false` in `config/test.toml`)
/// but kept in one place so it can't drift between test modules.
#[cfg(test)]
pub(crate) fn placeholder_today() -> NaiveDate {
    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()
}

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
    /// Colour for the UV gradient bar. `None` (zero UV, i.e. nighttime) uses
    /// the dashboard's own background colour so it blends into the page in
    /// any theme, rather than a fixed colour that would stand out against a
    /// dark background.
    pub fn to_colour(self, background_colour: &str) -> String {
        match self {
            Self::None => background_colour.to_string(),
            Self::Low => "green".to_string(),
            Self::Moderate => "yellow".to_string(),
            Self::High => "orange".to_string(),
            Self::VeryHigh => "red".to_string(),
            Self::Extreme => "purple".to_string(),
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
///   by concatenating the `svg_icons_directory` from the icon context with the icon name.
pub trait Icon {
    /// Returns the name of the icon
    fn icon_name(&self, ctx: &IconContext) -> String;

    /// Returns the path of the icon as a `String`.
    /// The path is constructed using the `svg_icons_directory` from the given
    /// context and the icon name obtained from `icon_name`.
    fn icon_path(&self, ctx: &IconContext) -> String {
        ctx.svg_icons_directory
            .join(Path::new(&self.icon_name(ctx)))
            .to_string_lossy()
            .to_string()
    }
}

impl Icon for SunPositionIconName {
    fn icon_name(&self, _ctx: &IconContext) -> String {
        self.to_string()
    }
}

impl Icon for HumidityIconName {
    fn icon_name(&self, _ctx: &IconContext) -> String {
        self.to_string()
    }
}

impl Icon for UVIndexIcon {
    fn icon_name(&self, _ctx: &IconContext) -> String {
        self.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configs::settings::DashboardSettings;

    mod uv_index_icon_boundaries {
        use super::*;

        #[test]
        fn zero_is_none() {
            assert!(matches!(UVIndexIcon::from(0), UVIndexIcon::None));
        }

        #[test]
        fn one_is_low() {
            assert!(matches!(UVIndexIcon::from(1), UVIndexIcon::Low));
        }

        #[test]
        fn two_is_still_low() {
            assert!(matches!(UVIndexIcon::from(2), UVIndexIcon::Low));
        }

        #[test]
        fn three_is_moderate() {
            assert!(matches!(UVIndexIcon::from(3), UVIndexIcon::Moderate));
        }

        #[test]
        fn five_is_still_moderate() {
            assert!(matches!(UVIndexIcon::from(5), UVIndexIcon::Moderate));
        }

        #[test]
        fn six_is_high() {
            assert!(matches!(UVIndexIcon::from(6), UVIndexIcon::High));
        }

        #[test]
        fn seven_is_still_high() {
            assert!(matches!(UVIndexIcon::from(7), UVIndexIcon::High));
        }

        #[test]
        fn eight_is_very_high() {
            assert!(matches!(UVIndexIcon::from(8), UVIndexIcon::VeryHigh));
        }

        #[test]
        fn ten_is_still_very_high() {
            assert!(matches!(UVIndexIcon::from(10), UVIndexIcon::VeryHigh));
        }

        #[test]
        fn eleven_is_extreme() {
            assert!(matches!(UVIndexIcon::from(11), UVIndexIcon::Extreme));
        }

        #[test]
        fn large_value_is_still_extreme() {
            assert!(matches!(UVIndexIcon::from(u16::MAX), UVIndexIcon::Extreme));
        }
    }

    #[test]
    fn uv_index_none_colour_falls_back_to_background() {
        assert_eq!(UVIndexIcon::None.to_colour("dark-blue"), "dark-blue");
    }

    #[test]
    fn uv_index_colours_by_variant() {
        assert_eq!(UVIndexIcon::Low.to_colour("bg"), "green");
        assert_eq!(UVIndexIcon::Moderate.to_colour("bg"), "yellow");
        assert_eq!(UVIndexIcon::High.to_colour("bg"), "orange");
        assert_eq!(UVIndexIcon::VeryHigh.to_colour("bg"), "red");
        assert_eq!(UVIndexIcon::Extreme.to_colour("bg"), "purple");
    }

    mod humidity_icon_name_boundaries {
        use super::*;

        #[test]
        fn forty_is_plain_humidity() {
            assert!(matches!(
                HumidityIconName::from(40),
                HumidityIconName::Humidity
            ));
        }

        #[test]
        fn forty_one_is_humidity_plus() {
            assert!(matches!(
                HumidityIconName::from(41),
                HumidityIconName::HumidityPlus
            ));
        }

        #[test]
        fn seventy_is_still_humidity_plus() {
            assert!(matches!(
                HumidityIconName::from(70),
                HumidityIconName::HumidityPlus
            ));
        }

        #[test]
        fn seventy_one_is_humidity_plus_plus() {
            assert!(matches!(
                HumidityIconName::from(71),
                HumidityIconName::HumidityPlusPlus
            ));
        }
    }

    #[test]
    fn icon_path_joins_svg_directory_with_icon_name() {
        let settings = DashboardSettings::load_test_config().unwrap();
        let ctx = IconContext::from_settings(&settings, placeholder_today());

        let path = SunPositionIconName::Sunrise.icon_path(&ctx);

        assert!(path.ends_with("sunrise.svg"));
        assert!(path.starts_with(&ctx.svg_icons_directory.to_string_lossy().to_string()));
    }
}
