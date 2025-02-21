use serde::Deserialize;
use std::env;

use config::{Config, ConfigError, Environment, File};
const CONFIG_DIR: &str = "./config";
const DEFAULT_CONFIG_NAME: &str = "default";

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Release {
    pub release_info_url: String,
    pub download_base_url: String,
    pub auto_update: bool,
    pub update_interval_days: i64,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Api {
    pub location: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Colours {
    pub background_colour: String,
    pub text_colour: String,
    pub x_axis_colour: String,
    pub y_left_axis_colour: String,
    pub y_right_axis_colour: String,
    pub temp_colour: String,
    pub feels_like_colour: String,
    pub rain_colour: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Misc {
    pub weather_data_store_path: String,
    pub template_path: String,
    pub modified_template_name: String,
    pub generated_png_name: String,
    pub svg_icons_directory: String,
    pub python_script_path: String,
    pub python_path: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct RenderOptions {
    pub saturation: f32,
    pub temp_unit: String,
    pub use_moon_phase_instead_of_clear_night: bool,
    pub x_axis_always_at_min: bool,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Debugging {
    pub disable_network_requests: bool,
    pub disable_png_output: bool,
    pub disable_drawing_on_epd: bool,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
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
/// # Examples
///
/// ```
/// let settings = DashboardSettings::new().unwrap();
/// println!("{:?}", settings);
/// ```
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

        let root = std::env::current_dir().map_err(|e| ConfigError::Message(e.to_string()))?;

        let default_config_path = root.join(CONFIG_DIR).join(DEFAULT_CONFIG_NAME);
        let run_mode_path = root.join(CONFIG_DIR).join(&run_mode);
        let local_config_path = root.join(CONFIG_DIR).join("local");

        // user config path is located at ~/.config/pi-inky-weather-epd.toml
        let home_dir = env::var("HOME").unwrap();
        let user_config_path = std::path::PathBuf::from(&home_dir)
            .join(".config")
            .join(format!("{}.toml", env!("CARGO_PKG_NAME")));

        let settings = Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name(default_config_path.to_str().unwrap()))
            // Add in user configuration file
            .add_source(File::with_name(user_config_path.to_str().unwrap()).required(false))
            // Add in the current environment file
            // Default to 'development' env
            // Note that this file is _optional_
            .add_source(File::with_name(run_mode_path.to_str().unwrap()).required(false))
            // Add in a local configuration file
            // This file shouldn't be checked in to git
            .add_source(File::with_name(local_config_path.to_str().unwrap()).required(false))
            // Add in settings from the environment (with a prefix of APP)
            // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
            .add_source(Environment::with_prefix("app"))
            .build()?;

        println!(
            "location: {:?}",
            settings.get_string("api.location").unwrap()
        );

        // You can deserialize (and thus freeze) the entire configuration as
        settings.try_deserialize()
    }
}
