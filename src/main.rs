use anyhow::Result;
use pi_inky_weather_epd::generate_weather_dashboard;
mod pimironi_image_py;
mod update;
use pimironi_image_py::pimironi_image_py;
use update::update;

fn main() -> Result<()> {
    generate_weather_dashboard()?;
    pimironi_image_py();
    update()
}
