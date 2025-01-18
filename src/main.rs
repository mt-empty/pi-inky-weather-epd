use anyhow::Result;
use pi_inky_weather_epd::generate_weather_dashboard;
mod update;
use update::update;

fn main() -> Result<()> {
    generate_weather_dashboard()?;
    update()
}
