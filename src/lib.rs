pub mod apis;
pub mod clock;
pub mod configs;
pub mod constants;
pub mod dashboard;
pub mod domain;
pub mod errors;
mod providers;
mod update;
pub mod utils;
pub mod weather;
pub mod weather_dashboard;

use crate::configs::settings::DashboardSettings;
use crate::weather_dashboard::generate_weather_dashboard;
use anyhow::Error;
use anyhow::Result;
use once_cell::sync::Lazy;
use update::update_app;

// Re-export for testing
pub use crate::weather_dashboard::generate_weather_dashboard_with_clock;

pub static CONFIG: Lazy<DashboardSettings> = Lazy::new(|| match DashboardSettings::new() {
    Ok(config) => {
        println!("## Configuration: {config:#?}");
        config
    }
    Err(e) => {
        eprintln!("Failed to load config: {e}");
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
