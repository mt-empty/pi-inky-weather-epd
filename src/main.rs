use anyhow::Result;
use pi_inky_weather_epd::run_weather_dashboard;

fn main() -> Result<()> {
    run_weather_dashboard()?;
    Ok(())
}
