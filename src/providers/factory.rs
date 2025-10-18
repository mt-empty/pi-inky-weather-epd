use crate::{
    configs::settings::Providers,
    providers::{bom::BomProvider, open_meteo::OpenMeteoProvider, WeatherProvider},
    CONFIG,
};

pub fn create_provider() -> anyhow::Result<Box<dyn WeatherProvider>> {
    let cache_path = CONFIG.misc.weather_data_cache_path.clone();

    match CONFIG.api.provider {
        Providers::Bom => Ok(Box::new(BomProvider::new(cache_path))),
        Providers::OpenMeteo => Ok(Box::new(OpenMeteoProvider::new(cache_path))),
    }
}
