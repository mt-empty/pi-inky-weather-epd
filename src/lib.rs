pub mod apis;
pub mod clock;
pub mod configs;
pub mod constants;
pub mod dashboard;
pub mod domain;
pub mod errors;
mod logger;
pub mod providers;
pub mod update;
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
pub use crate::weather_dashboard::generate_weather_dashboard_injection;
pub use clock::{Clock, FixedClock, SystemClock};

pub static CONFIG: Lazy<DashboardSettings> = Lazy::new(|| match DashboardSettings::new() {
    Ok(config) => {
        config.print_config();
        config
    }
    Err(e) => {
        logger::error(format!("Failed to load config: {e}"));
        std::process::exit(1);
    }
});

pub fn generate_weather_dashboard_wrapper() -> Result<(), Error> {
    generate_weather_dashboard()
}

pub fn run_weather_dashboard() -> Result<(), anyhow::Error> {
    logger::app_start("Pi Inky Weather Display", env!("CARGO_PKG_VERSION"));

    logger::section("Generating weather dashboard");
    generate_weather_dashboard_wrapper()?;

    if CONFIG.release.update_interval_days.into_inner() > 0 {
        logger::section("Checking for updates");
        update_app()?;
    };

    logger::app_end();
    Ok(())
}

/// Run weather dashboard with a custom clock (for simulation/testing)
pub fn run_weather_dashboard_with_clock(clock: &dyn Clock) -> Result<(), anyhow::Error> {
    logger::app_start("Pi Inky Weather Display", env!("CARGO_PKG_VERSION"));

    logger::section("Generating weather dashboard (simulation mode)");
    let input_template_name = &CONFIG.misc.template_path;
    let output_svg_name = &CONFIG.misc.generated_svg_name;
    generate_weather_dashboard_injection(clock, input_template_name, output_svg_name)?;

    // Skip auto-update in simulation mode
    logger::detail("Skipping auto-update check in simulation mode");

    logger::app_end();
    Ok(())
}
