use anyhow::Result;

#[cfg(not(feature = "cli"))]
use pi_inky_weather_epd::run_weather_dashboard;

// CLI features only available when 'cli' feature is enabled (for simulation/testing)
#[cfg(feature = "cli")]
mod cli {
    use anyhow::Result;
    use clap::Parser;
    use pi_inky_weather_epd::{
        clock::FixedClock, run_weather_dashboard, run_weather_dashboard_with_clock,
    };

    /// Pi Inky Weather Display - Generate weather dashboards for e-paper displays
    #[derive(Parser, Debug)]
    #[command(name = "pi-inky-weather-epd")]
    #[command(version, about, long_about = None)]
    pub struct Args {
        /// Simulate mode: Use a fixed timestamp (RFC3339 format, e.g., "2025-12-26T09:00:00Z")
        /// When provided, the dashboard will be generated as if it's this time.
        /// Useful for generating multiple dashboards at different times for testing.
        #[arg(long, value_name = "TIMESTAMP")]
        pub simulate_time: Option<String>,
    }

    pub fn run() -> Result<()> {
        let args = Args::parse();

        if let Some(timestamp) = args.simulate_time {
            let fixed_clock = FixedClock::from_rfc3339(&timestamp).map_err(|e| {
                anyhow::anyhow!(
                    "Invalid timestamp format: {}. Expected RFC3339 format like '2025-12-26T09:00:00Z'",
                    e
                )
            })?;
            run_weather_dashboard_with_clock(&fixed_clock)?;
        } else {
            run_weather_dashboard()?;
        }

        Ok(())
    }
}

#[cfg(feature = "cli")]
fn main() -> Result<()> {
    cli::run()
}

#[cfg(not(feature = "cli"))]
fn main() -> Result<()> {
    run_weather_dashboard()?;
    Ok(())
}
