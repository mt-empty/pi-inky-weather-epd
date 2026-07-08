use crate::{
    configs::settings::{DashboardSettings, Providers},
    providers::{bom::BomProvider, open_meteo::OpenMeteoProvider, WeatherProvider},
};

pub fn create_provider(settings: &DashboardSettings) -> anyhow::Result<Box<dyn WeatherProvider>> {
    let cache_path = settings.misc.weather_data_cache_path.clone();

    match settings.api.provider {
        Providers::Bom => Ok(Box::new(BomProvider::new(cache_path))),
        Providers::OpenMeteo => Ok(Box::new(OpenMeteoProvider::new(cache_path))),
    }
}
