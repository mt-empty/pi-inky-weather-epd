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
use update::update_app;

// Re-export for testing
pub use crate::weather_dashboard::generate_weather_dashboard_injection;
pub use crate::weather_dashboard::render_svg_to_png;
pub use clock::{Clock, FixedClock, SystemClock};

pub fn generate_weather_dashboard_wrapper(settings: &DashboardSettings) -> Result<(), Error> {
    generate_weather_dashboard(settings)
}

pub fn run_weather_dashboard(settings: &DashboardSettings) -> Result<(), anyhow::Error> {
    logger::init(settings.dev.enable_debug_logs, settings.misc.timezone);
    logger::init_file_log();
    logger::app_start("Pi Inky Weather Display", env!("CARGO_PKG_VERSION"));
    settings.print_config();

    logger::section("Generating weather dashboard");
    generate_weather_dashboard_wrapper(settings)?;

    if settings.release.update_interval_days.into_inner() > 0 {
        logger::section("Checking for updates");
        update_app(settings, &SystemClock)?;
    };

    logger::app_end();
    Ok(())
}

/// Run weather dashboard with a custom clock (for simulation/testing)
pub fn run_weather_dashboard_with_clock(
    settings: &DashboardSettings,
    clock: &dyn Clock,
) -> Result<(), anyhow::Error> {
    logger::init(settings.dev.enable_debug_logs, settings.misc.timezone);
    logger::init_file_log();
    logger::app_start("Pi Inky Weather Display", env!("CARGO_PKG_VERSION"));
    settings.print_config();

    logger::section("Generating weather dashboard (simulation mode)");
    let input_template_name = &settings.misc.template_path;
    let output_svg_name = &settings.misc.generated_svg_name;
    generate_weather_dashboard_injection(settings, clock, input_template_name, output_svg_name)?;

    // Skip auto-update in simulation mode
    logger::detail("Skipping auto-update check in simulation mode");

    logger::app_end();
    Ok(())
}
