use super::validation::*;
use nutype::nutype;
use serde::Deserialize;
use std::{env, fmt, path::PathBuf};
use strum_macros::Display;
use url::Url;

use config::{Config, ConfigError, Environment, File};
const CONFIG_DIR: &str = "./config";
const DEFAULT_CONFIG_NAME: &str = "default";

#[derive(Debug, Deserialize, PartialOrd, PartialEq, Clone, Copy, Display)]
#[serde(rename_all = "snake_case")]
pub enum Providers {
    Bom,
    OpenMeteo,
}

#[derive(Debug, Deserialize, PartialOrd, PartialEq, Clone, Copy, Display)]
#[serde(rename_all = "UPPERCASE")]
pub enum TemperatureUnit {
    #[strum(serialize = "C")]
    C,
    #[strum(serialize = "F")]
    F,
}

#[nutype(
    sanitize(trim),
    validate(with = is_valid_colour, error = ValidationError),
    derive(Debug, Deserialize, PartialEq, Clone)
)]
pub struct Colour(String);

impl fmt::Display for Colour {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.clone().into_inner())
    }
}

#[nutype(
    sanitize(trim, lowercase),
    validate(len_char_min = 6, len_char_max = 6),
    derive(Debug, Deserialize, PartialEq, Clone, AsRef)
)]
pub struct GeoHash(String);

impl fmt::Display for GeoHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.clone().into_inner())
    }
}

#[nutype(
    sanitize(),
    validate(greater_or_equal = 0),
    derive(Debug, Deserialize, PartialEq, Clone, AsRef, Copy)
)]
pub struct UpdateIntervalDays(i32);

impl fmt::Display for UpdateIntervalDays {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.into_inner())
    }
}

#[nutype(
    sanitize(),
    validate(with = is_valid_longitude, error = ValidationError),
    derive(Debug, Deserialize, PartialEq, Clone, Copy, AsRef)
)]
pub struct Longitude(f64);

impl fmt::Display for Longitude {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.into_inner())
    }
}

#[nutype(
    sanitize(),
    validate(with = is_valid_latitude, error = ValidationError),
    derive(Debug, Deserialize, PartialEq, Clone, Copy, AsRef)
)]
pub struct Latitude(f64);

impl fmt::Display for Latitude {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.into_inner())
    }
}

#[derive(Debug, Deserialize)]
pub struct Release {
    pub release_info_url: Url,
    pub download_base_url: Url,
    pub update_interval_days: UpdateIntervalDays,
}

#[derive(Debug, Deserialize)]
pub struct Api {
    pub provider: Providers,
    pub longitude: Longitude,
    pub latitude: Latitude,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Colours {
    pub background_colour: Colour,
    pub text_colour: Colour,
    pub x_axis_colour: Colour,
    pub y_left_axis_colour: Colour,
    pub y_right_axis_colour: Colour,
    pub actual_temp_colour: Colour,
    pub feels_like_colour: Colour,
    pub rain_colour: Colour,
}

#[derive(Debug, Deserialize)]
pub struct Misc {
    pub weather_data_cache_path: PathBuf,
    pub template_path: PathBuf,
    pub generated_svg_name: PathBuf,
    pub generated_png_name: PathBuf,
    pub svg_icons_directory: PathBuf,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RenderOptions {
    pub temp_unit: TemperatureUnit,
    pub use_moon_phase_instead_of_clear_night: bool,
    pub x_axis_always_at_min: bool,
    pub use_gust_instead_of_wind: bool,
}

#[derive(Debug, Deserialize)]
pub struct Debugging {
    pub disable_weather_api_requests: bool,
    pub disable_png_output: bool,
    pub allow_pre_release_version: bool,
}

#[derive(Debug, Deserialize)]
pub struct DashboardSettings {
    pub release: Release,
    pub api: Api,
    pub colours: Colours,
    pub misc: Misc,
    pub render_options: RenderOptions,
    pub debugging: Debugging,
}

/// Dashboard settings.
///
/// # Fields
///
/// * `release` - Release settings.
/// * `api` - API settings.
/// * `colours` - Colour settings.
/// * `misc` - Miscellaneous settings.
/// * `render_options` - Render options.
/// * `debugging` - Debugging settings.
///
/// # Errors
///
/// Returns an error if the configuration cannot be loaded.
///
/// # Panics
///
/// Panics if the configuration file is not found.
impl DashboardSettings {
    pub(crate) fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
        let is_test_mode = run_mode == "test";

        let root = std::env::current_dir().map_err(|e| ConfigError::Message(e.to_string()))?;

        let default_config_path = root.join(CONFIG_DIR).join(DEFAULT_CONFIG_NAME);
        let development_config_path = root.join(CONFIG_DIR).join("development");
        let local_config_path = root.join(CONFIG_DIR).join("local");
        let test_config_path = root.join(CONFIG_DIR).join("test");

        // user config path is located at ~/.config/pi-inky-weather-epd.toml
        let home_dir = env::var("HOME").unwrap();
        let user_config_path = std::path::PathBuf::from(&home_dir)
            .join(".config")
            .join(env!("CARGO_PKG_NAME"));

        let mut config_builder = Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name(default_config_path.to_str().unwrap()))
            // Add in user configuration file
            .add_source(File::with_name(user_config_path.to_str().unwrap()).required(false));

        // If running tests (RUN_MODE=test), load test.toml and skip development/local
        // Otherwise, load development.toml and local.toml
        if is_test_mode {
            config_builder = config_builder
                .add_source(File::with_name(test_config_path.to_str().unwrap()).required(false));
        } else {
            config_builder = config_builder
                // Add in development configuration file
                .add_source(File::with_name(development_config_path.to_str().unwrap()).required(false))
                // Add in local configuration file (for dev overrides, not checked into git)
                .add_source(File::with_name(local_config_path.to_str().unwrap()).required(false));
        }

        let settings = config_builder
            // Add in settings from the environment (with a prefix of APP)
            // Eg.. `APP_API__PROVIDER=open_meteo` would set the `api.provider` key
            // Note: Single underscore _ separates prefix from key, double __ for nesting
            .add_source(
                Environment::with_prefix("APP")
                    .prefix_separator("_")  // Separator between prefix and key (APP_api)
                    .separator("__")        // Separator for nested keys (api__provider)
                    .try_parsing(true),             // Parse values to correct types
            )
            .build()?;
        let final_settings: Result<DashboardSettings, ConfigError> = settings.try_deserialize();

        // Validate the settings after deserializing
        if let Err(error) = &final_settings {
            return Err(ConfigError::Message(format!(
                "Configuration validation failed: {error:?}"
            )));
        }

        final_settings
    }
}
