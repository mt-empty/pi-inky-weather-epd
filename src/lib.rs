mod apis;
mod configs;
pub mod constants;
mod dashboard;
pub mod domain;
mod errors;
mod providers;
mod update;
mod utils;
pub mod weather;
pub mod weather_dashboard;

use crate::configs::settings::DashboardSettings;
use crate::weather_dashboard::generate_weather_dashboard;
use anyhow::Error;
use anyhow::Result;
use once_cell::sync::Lazy;
use update::update_app;

// #[cfg(debug_assertions)]
// mod dev;

// #[cfg(debug_assertions)]
// use dev::create_striped_png;

pub static CONFIG: Lazy<DashboardSettings> = Lazy::new(|| match DashboardSettings::new() {
    Ok(config) => {
        println!("## Configuration: {:#?}", config);
        config
    }
    Err(e) => {
        eprintln!("Failed to load config: {}", e);
        std::process::exit(1);
    }
});

pub fn generate_weather_dashboard_wrapper() -> Result<(), Error> {
    generate_weather_dashboard()
}

pub fn run_weather_dashboard() -> Result<(), anyhow::Error> {
    println!("# Generating weather dashboard...");
    generate_weather_dashboard_wrapper()?;

    if CONFIG.release.update_interval_days.into_inner() > 0 {
        println!("## Checking for updates...");
        update_app()?;
    };
    Ok(())
}
