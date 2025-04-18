mod apis;
mod configs;
pub mod constants;
mod dashboard;
mod errors;
mod update;
mod utils;
pub mod weather;
pub mod weather_dashboard;

use crate::configs::settings::DashboardSettings;
use crate::weather_dashboard::generate_weather_dashboard;
use anyhow::Error;
use anyhow::Result;
use lazy_static::lazy_static;
use update::update_app;

// #[cfg(debug_assertions)]
// mod dev;

// #[cfg(debug_assertions)]
// use dev::create_striped_png;

lazy_static! {
    pub static ref CONFIG: DashboardSettings =
        DashboardSettings::new().expect("Failed to load config");
}

pub fn generate_weather_dashboard_wrapper() -> Result<(), Error> {
    generate_weather_dashboard()
}

pub fn run_weather_dashboard() -> Result<(), anyhow::Error> {
    generate_weather_dashboard_wrapper()?;
    if CONFIG.release.update_interval_days > 0 {
        update_app()?;
    };
    Ok(())
}
