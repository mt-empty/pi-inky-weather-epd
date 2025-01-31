use anyhow::Result;
use pi_inky_weather_epd::{generate_weather_dashboard, CONFIG};
mod pimironi_image_py;
mod update;
use pimironi_image_py::invoke_pimironi_image_script;
use update::update_app;

fn main() -> Result<()> {
    generate_weather_dashboard()?;
    invoke_pimironi_image_script()?;
    if CONFIG.release.auto_update {
        update_app()?;
    }
    Ok(())
}
